#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要 cargo：缺了就在单测那一格判红、复跑那一格由它调的 scripts/experiment-apparatus/replay.sh 编实验二进制时判红，不交给 gate.sh 预判；此外只要跑门禁本身就要的 bash、git、python3
# gate-stage: research 工作区的构建与单测；入库的实验数今天还复现得出来吗
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: harness-model-differential-and-scenarios.sh 它也经内存包装跑 release 的 cargo test，但编的是仓根的 crates 工作区、判的是模型对拍那几段；这里编的是 research 工作区，两个工作区互相看不到对方，编译产物不共用，并进来反而要多编一份
# gate-similar: code-source-discipline.sh 它的实验源码纪律那一格静态判 research 的实验二进制与变异表（锚点、每个实验有变异表），不编译、不跑；这里真编、真跑单测与复跑，判的是编得过、跑得出同样的数；mutation-tables-compile 格枚举的表与它同一个来源，逐条施加替换文真编（只 cargo check），判的是每条替换文今天还编得过
# gate-similar: doc-experiments.sh 它判实验页与产物登记（索引行、产物写回、产物落在仓里），读的是 kb 与 research/results/ 的文字；这里把产物重跑一遍逐字节比，读的是实验二进制的输出
# gate-similar: code-tooling.sh 研究脚本的 --selftest 都在它的 research-script-selftests 格跑（不要 research 工作区先编好、经 light-selftest-guard.py 盯着）；这里只编 research 工作区、跑它的单测与复跑，一份研究脚本的自证都不跑
#
# gate-cell: research-unit-tests research 工作区编得过、单测全绿
# gate-cell: wallclock-asserts-registered 实验单测里按挂钟判的断言都有登记位（一行注释），没登记的判红
# gate-cell: experiment-replay 入库的纯模型实验重跑一遍，与 research/results/ 里的产物逐字节相同
# gate-cell: mutation-tables-compile 研究侧每张变异表逐条施加替换文只编译，一条编不过就红
#   research-unit-tests  cargo build --release 编整个 research 工作区（每趟照编：「编得过」判的是整个目标，闭包够不到的死码在 deny 警告下也编不过，
#                        单元闭包罩不住；增量编，没改动时 cargo 判新鲜），再 cargo test --release。
#                        上游「构建与单测」只编 crates/，kb 里实测结论背后的实验二进制只有这一格编。
#                        按条复用：每条测试函数（lib、各 bin、tests/ 里带 #[test] 的）一条判定，键是它的单元键（scripts/gate/admission.py test-plan，
#                        闭包根 scripts/experiment-apparatus、release 档、工具链、解析出的依赖版本，判法 scripts/gate/lib_libtest_result_lines.py 的 result_line），
#                        键相同的经 cargo test -- --exact --skip <全名> 跳过，一条都不用现判就不起 cargo test；判绿的照键记进
#                        <git common-dir>/singlefs-unit-verdicts/checker-tier-research-build-and-replay.sh:research-unit-tests/。拒算的照跑、不记；
#                        成功句报「单测按条：现判 N、复用 M、拒算 R、本次未跑 K」，红句另列红的那几条这一趟为什么现判（变了的单元）。
#                        按条复用失灵（test-plan 退非 0、退 0 而没打 PLAN、它或 verdict-record 的复用哨兵报「复用失灵」「判定不稳」「强制现判没起作用」）：
#                        判法只在 scripts/gate/admission.py 的 plan-failure（文件头「按条复用失灵」）；它判失灵，这一格照旧全部现判跑完、结论判红。
#                        样本（被判的根不是这道阶段所在的仓）另放了 .unit-verdict-sample/prior/（上一版的树）的，开跑前先拿上一版算键、照判绿记进样本自己的判定库。
#                        登记了的挂钟断言按名跳过（C465：门禁自己的并发让按挂钟判的断言误红）：源码里测试函数上带 // wallclock-assert: 登记位（一行注释，装置不加任何代码），
#                        判法 scripts/gate/wallclock-assert-lint.py --list 逐行打「<文件>TAB<函数名>」，这一格按登记名逐个给 cargo test 传 --skip <函数名>
#                        （按条复用算得出时整条命令带 --exact，登记名换成计划里末段是它的那几条全名；算不出时照旧 cargo test --release -- --skip <名> --skip <名>…，
#                        按子串跳）。不设环境变量、不加 --show-output。
#                        跑完从输出里找 ^test .*<函数名> ... 行：登记的名字出现了判红（登记了却跑了：--skip 没生效）；
#                        每个登记名逐条往 $GATE_NOT_RUN_FILE 报「挂钟断言 <函数名> 本次未跑」，成功句报「跳过挂钟断言 K 条（逐个列名）」，不记它们为绿、不判红。
#                        登记位写坏时判法 --list 退 1，这一格不数、不传 --skip，那一错归 wallclock-asserts-registered 格。
#                        要验这些断言：不带 --skip，在不并发的场合单独跑 cargo test。
#   wallclock-asserts-registered  python3 scripts/gate/wallclock-assert-lint.py：scripts/experiment-apparatus/*.rs 的 #[test] 里按挂钟判的断言要带 // wallclock-assert: 登记位
#                        （或怕不着负载的 // wallclock-assert-load-safe:），缺了判红、点名文件:行号；带了而断言里没有挂钟量也判红；登记名是别的未登记测试名的子串
#                        （--skip 是子串匹配，会连那一条一起跳过）也判红。判法、登记位写法在判法脚本文件头；
#                        判法立之前就在仓里还没复核的在 .claude/gate.d/wallclock-assert-lag.tsv（只缩不涨），成功句另报条数，并往 $GATE_NOT_RUN_FILE 报一行。
#   experiment-replay    不整格问复用、不比树、不比 HEAD：派给 scripts/experiment-apparatus/replay.sh，它按行复用（下面「要跑时」那一段）。
#                        replay.sh 退 77（C350：整趟全是计时行的负载关，本次不判）不当红：这一格报「复跑整趟本次不判（负载高）」、往 $GATE_NOT_RUN_FILE 报一行、退 77；别的退出码照旧（非 0 判红）。
#                        派给 scripts/experiment-apparatus/replay.sh 的是表里的 exact 行减掉 SLOW；SLOW 里的与表里每一行 timing 都逐个列成「没派」。
#                        对不上不一定是代码坏了，也可能是产物该更新，两种都要人判，所以判红。
#                        replay.sh 按行复用（它文件头「按行复用」：每一行的行键是装置入口的单元闭包、参数、GPU 开关与特性集、期望产物内容、工具链、
#                        解析出的依赖版本，算法在 scripts/gate/admission.py 文件头「复跑表每一行的行键」；键相同、判过的那一行不跑，全部行都复用的一趟不起编译、不清场）：
#                        这一格读它末尾那一行 REPLAY_TOTAL，报「复跑：现判 N 行、复用 M 行、拒算 R 行、本次未跑 K 行」（本次未跑 = 已归档只跑不比 + 负载高没判）；
#                        没有那一行（老的或假的 replay.sh）报复用数取不出，不写 0。已归档只跑不比的行由 replay.sh 逐行往 $GATE_NOT_RUN_FILE 报。
#                        SINGLEFS_GATE_FULL=1 原样带下去（replay.sh 照它只记不读）。
#   mutation-tables-compile  研究侧变异表（scripts/mutation/tables/<名>.tsv）逐张、逐条施加替换文只编译：每张调
#                        scripts/mutation/mutate.sh --compile-only <bin> <名>.rs <表>（判法、cargo 跑什么、退出码都在它文件头「只编不跑」；
#                        每个 cargo 带的 RUSTFLAGS=…--cap-lints warn 在它文件头「deny 警告」，这里不另加）；
#                        有一条 ⏭（编不过：平常跑里就是一条无效变异）整格红，逐条点名表与条目、打第一行 error。
#                        枚举对象与 code-source-discipline 的 mutation-tables 格同一个来源：scripts/mutation/tables/ 下每张 .tsv 对 scripts/experiment-apparatus/ 下同名的 .rs；
#                        bin 名经 mutate.sh --binary-of <名>.rs 取（与它核对 bin 的读法同一份）。没有同名 .rs 的表（shell 探针、Python 驱动、crates 那一侧的）
#                        取不出（bin、源文件、表）三元组，逐张点名、往 $GATE_NOT_RUN_FILE 报本次未跑、不算绿；一张都判不了退 77。
#                        按条复用归 mutate.sh（它文件头「按条入库」：每条变异一条判定，键是原文、替换文、包住原文的单元闭包、cargo 参数与特性、工具链、依赖版本，
#                        库在 git common-dir 底下的 singlefs-mutate-library/，不进 /tmp）：这一格不另建库、不整格问复用；一张表的条全部复用时 mutate.sh 一条都不编。
#                        这一格读每张表末尾「计数：」那一行，逐张报「现判 N 条、复用 M 条、拒算 R 条」，成功句报合计与「本次未跑 K 张表」；
#                        编不过的那一条另报 mutate.sh 写的「现判：[<名>] <原因>」（键对不上时是变了的单元）。SINGLEFS_GATE_FULL=1 时给 mutate.sh 带 --full（只写不读）。
#                        编译目录一个、这一趟自己的：这一格的临时目录（mktemp -d）里的 target/，各表串行增量编（编译产物，不是判定库），收尾连同临时目录删
#                        （用户 2026-10-23 定案「自己用完自己删」，不跨轮复用）。
#                        挂了 required-features 的 bin 由 mutate.sh 带上 --features；E162 的 rocksdb 要的 BINDGEN_EXTRA_CLANG_ARGS 没设时取
#                        scripts/crash/crash-amplification-lib.sh 的 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS（与 checker-tier-crash-point-replay 带特性那一趟同一处）。
# 格结构经 lib/stage-cells.sh。
#
# 读的环境变量：
#   GATE_RESEARCH_BUILD_MEMORY_MAX  cargo test 经 scripts/hosts/run-with-memory-cap.sh 跑的内存上限，没设取 8G；谁跑这一道外面都不再包一层。
#                                   包装退 245 或 250–254 不是单测的判定，单独判红、单独给出路。cargo build 不跑编出来的代码，不经包装。
#   GATE_REPLAY_FULL=1              复跑那一格连 SLOW 与 timing 行一起跑
#   SINGLEFS_GATE_FULL=1            只写不读：research-unit-tests 每条现判、照键记；replay.sh 照它只记不读；mutation-tables-compile 给 mutate.sh 带 --full
#
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh [项目根]                                三格都跑
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --list                                  逐行打格名与判什么，不跑格
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests [项目根]     只跑单测那一格
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check wallclock-asserts-registered [项目根]  只跑挂钟断言登记位那一格
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --list-items [项目根]                    逐行打 replay.sh 表里的实验号、exact 还是 timing、在不在 SLOW 里，不编、不跑；
#                                                                                                         末尾另打一行复跑表行键对不上的行数（admission.py replay-row-inputs，只算键）
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check experiment-replay --item E142 [项目根]   只复跑点名的实验
#     --item 给几次取并集，timing 与 SLOW 也照跑；成功行写明只跑了点名的几个，不是整格结论；实验号不在表里退 2
#   bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check mutation-tables-compile [项目根]  只跑变异表只编译那一格
#
# 样本：fixtures/checker-tier-research-build-and-replay.sh/ 下十九份，各放 .gate-cells 只点名它判的那一格：
#   research-unit-tests-red    research 工作区里有一条单测红着的小仓，判红
#   research-unit-tests-green  一条单测全绿、scripts/ 下一份脚本都没有，判绿，成功行报一个测试批次
#   green                      带假 replay.sh 的小仓，表里一行快的 exact、一行 SLOW 里的 exact、一行 SLOW 里只有 timing 的；
#                              快的那行派给 replay.sh，另两行逐个列进「没派」
#   red                        表一行都没有，判红
#   mutation-tables-compile-red    最小 research 工作区（一个 bin、warnings = "deny"），一张两条的表里一条替换文编不过（引了没有的名字），判红、点名表与那一条
#   mutation-tables-compile-green  同形的工作区，两条替换文都编得过，另有一张没有同名 .rs 的表（逐张列进本次未跑）；
#                              样本根放 .mutation-tables-compile-sample/run-twice：按条判定库放这一格自己的临时目录，接连判两趟，第二趟两条都复用、一条都不编
#   mutation-tables-compile-row-key  同形的工作区、run-twice，另放 .mutation-tables-compile-sample/second-round/（第二趟之前盖上：表里第 2 条的替换文改一个字）：
#                              第二趟第 1 条复用、第 2 条现判（对照样本 T14 的 :2），判绿
#   mutation-tables-compile-deny-green  同形的工作区（warnings = "deny"），一条替换文只让一个函数再没人调、留下死代码：deny 下编不过，
#                              mutate.sh 给每个 cargo 带的 --cap-lints warn 压成警告，判绿（两条都编得过）
#   wallclock-asserts-registered-red    scripts/experiment-apparatus/ 下一份 .rs 里有一个按挂钟判的断言（.elapsed() 比大小）的测试，没登记位，判红、点名文件:行号
#   wallclock-asserts-registered-green  同形的一份 .rs：一个带 // wallclock-assert: 的测试、一个带 // wallclock-assert-load-safe: 的测试，判绿并报登记 1 条、load-safe 1 条
#   replay-load-skipped        带假 replay.sh（退 77）的小仓：复跑那一格认成本次未跑，整道退 77、输出里有「复跑整趟本次不判（负载高）」；改前这一格对 77 退 1
#   experiment-replay-reused   带假 replay.sh 的小仓，末行打一行 REPLAY_TOTAL（现判 1 行、复用 2 行、拒算 0 行、已归档只跑不比 1 行）：这一格照它报复用数，判绿
#   research-unit-tests-unit-key  research 小工作区（一个 lib，测试模块里三条单测），setup.sh git init、照这一版拷一份上一版进 .unit-verdict-sample/prior/、
#                              在上一版里删掉第三条单测（对照样本 N06：测试模块加一条单测，别的不作废）：现判 1、复用 2，cargo test 带 --exact --skip 那两条，判绿
#   research-unit-tests-plan-exit-red  按条复用失灵：同 research-unit-tests-green 的小仓，样本根放 .unit-verdict-sample/plan-failure 写 exit-1（test-plan 退 1），
#                              照旧全部现判、单测都过，判红、点名 test-plan 退 1
#   research-unit-tests-plan-sentinel-red  同形，写 sentinel-reuse-failed（test-plan 的复用哨兵报复用失灵），判红、点名哨兵那一句
#   research-unit-tests-archived-not-run  research 小工作区（一个 lib、两条单测，setup.sh git init、没有上一版），一条单测往标准输出写一行
#                              NOT_RUN_ARCHIVED_PRODUCT test=lib::tests::<它自己> reason=…（读不到留存产物）：这一格判绿、那一条报本次未跑、不记库（RECORD recorded=1）
#   research-unit-tests-archived-not-run-unmatched-red  同形，那一行的 test= 写成计划里没有的名字：判红、点名对不上的那一条
#   experiment-replay-inputs-not-run  带假 replay.sh 的小仓：打两行本次未跑与 REPLAY_TOTAL（not_run=2、load_skipped=0）后退 77，
#                              这一格报「输入不在 2 行」「复跑整趟本次不判（要读的产物全不在）」、整道退 77
#   experiment-replay-rocksdb-entry-refused  带假 replay.sh（同 experiment-replay-reused）的小仓，crates/singlefs-checker-tier/Cargo.toml 依赖 rocksdb：
#                              起 replay.sh 之前 rocksdb-prebuilt.py entry 判红（样本根没有 Cargo.lock，进程环境有没有取值都红），这一格退 1、replay.sh 没起
#   mutation-tables-compile 的样本里按条判定库与编译目录都是这一格自己的临时目录，跑完删；run-twice 那两份第二趟读第一趟记的。mutation-tables-compile 四份要真 cargo（经 mutate.sh cargo check 一个小 bin）；
#   research-unit-tests-unit-key 的判定库在样本自己的 git common-dir 里（setup.sh git init），跑完随临时目录删；
#   wallclock-asserts-registered 两份只读样本里的 .rs，不起 cargo。
# 弄坏开关归共用库：STAGE_CELLS_BREAK=red-swallowed 让三份红样本整道退 0；=marker-ignored 让每份样本三格都跑，
#   research-unit-tests-green 里复跑那一格对着没有表的 replay.sh 判红。
# 这一道自己的弄坏开关 RESEARCH_BUILD_AND_REPLAY_BREAK=<项>（逐项写开关名、弄坏哪一处、打开后哪份样本红）：
#   invalid-rows-ignored  mutation-tables-compile 不看 mutate.sh --compile-only 打的 ⏭，退 1 的表也当绿（改前没有这一格的样子）
#       mutation-tables-compile-red（期望退 1、输出里有「[两条之二_引了没有的名字] 编不过」，实得退 0）
#   no-cap-lints  mutation-tables-compile 起 mutate.sh --compile-only 时，PATH 前面放一个垫片 cargo（这一格自己的临时目录里）：它先 env -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS 再交给真 cargo，
#       mutate.sh 带的 --cap-lints warn 不管放在 RUSTFLAGS 还是 CARGO_ENCODED_RUSTFLAGS 里都到不了 rustc（改前的跑法）。
#       不靠设 CARGO_ENCODED_RUSTFLAGS：mutate.sh 看见它就把 cap 接进去，那一招顶不掉了。mutate.sh 自己的 MUTATE_BREAK 只在它的 --selftest 起的子进程里认，门禁里设了它 mutate.sh 退 2（要保留），所以样本上证红用这一个
#       mutation-tables-compile-deny-green（期望退 0，实得退 1、输出里有「[两条之二_留死代码_不再调用加倍] 编不过：error: function `double` is never used」）
#   skip-filter-dropped  research-unit-tests 起 cargo test 时不传 --skip：登记了的挂钟断言在门禁的负载下照跑，输出里出现它们的 test … ok / FAILED 行，这一格红
#       「登记了却跑了：--skip 没生效」。样本档里 research-unit-tests 两份样本的小仓没有登记位，登记数是 0、没有 --skip 可丢，看不出这一项，样本上证不了红；只靠真仓真跑：
#       SINGLEFS_HEAVY_TESTS=commit RESEARCH_BUILD_AND_REPLAY_BREAK=skip-filter-dropped bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests
#       必须退 1、输出里有「登记了却跑了」。wallclock-asserts-registered 格不起 cargo，不受影响。
#   archived-not-run-ignored  research-unit-tests 不认单测写的 NOT_RUN_ARCHIVED_PRODUCT 行（改前的样子）：读不到留存产物的那一条照判绿记库
#       research-unit-tests-archived-not-run（输出里找不到「! 本次未跑：lib::tests::reads_an_archived_product_that_is_not_there」，实得 RECORD recorded=2）、
#       research-unit-tests-archived-not-run-unmatched-red（期望退 1，实得退 0）
#   replay-not-run-uncounted  experiment-replay 不读 REPLAY_TOTAL 的 not_run=（改前的样子）：输入不在的行不进本次未跑的数，整趟退 77 照说负载高
#       experiment-replay-inputs-not-run（输出里找不到「复跑整趟本次不判（要读的产物全不在）」）
#   replay-reuse-unreported  experiment-replay 不读 replay.sh 的 REPLAY_TOTAL 行，复用数不报（改前的样子）
#       experiment-replay-reused（输出里找不到「复跑：现判 1 行、复用 2 行、拒算 0 行、本次未跑 1 行」）
#   row-key-with-product-name  experiment-replay 给 replay.sh 设 ADMISSION_BREAK=row-key-with-product-name：复跑表每一行的行键另加产物文件名（改名就作废）。
#       样本上证不了（要真的 replay.sh 与 research 工作区）；同一个开关在 python3 scripts/gate/admission.py --selftest 的「复跑表行键」R03 那一格转红
#       （只改产物文件名、内容不变的那一行期望复用、实得现判）
#   mutation-compile-key-whole-table  mutation-tables-compile 给 mutate.sh 的按条判定库按整张表的 sha256 分开放（改前按表一项的样子：表里改一条，整张表的条都作废）
#       mutation-tables-compile-row-key（输出里找不到第二趟的「现判 1 条、复用 1 条」：第 1 条没改也被判成现判）
#   unit-tests-without-library  research-unit-tests 的计划带 --full（一条都不复用，改前没有库的样子）
#       research-unit-tests-unit-key（输出里找不到「现判 1、复用 2」：只加了一条单测，另两条也现判）
#   rocksdb-entry-unchecked  experiment-replay 起 replay.sh 之前不调 scripts/hosts/rocksdb-prebuilt.py entry（改前的样子）
#       experiment-replay-rocksdb-entry-refused（期望退 1、输出里有「rocksdb-prebuilt entry：」，实得退 0：假 replay.sh 照起了）
# admission.py 的 ADMISSION_BREAK=plan-failure-not-red（按条复用失灵也不判红，改前退回全部现判的样子）：
#       research-unit-tests-plan-exit-red、research-unit-tests-plan-sentinel-red（期望退 1，实得退 0）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"
STAGE_REPOSITORY="$(cd "$STAGE_DIR/../.." && pwd)"
STAGE_NAME="$(basename "$0")"
source "$STAGE_DIR/lib/stage-cells.sh"

stage_cell research-unit-tests cell_research_unit_tests "research 工作区编得过、单测全绿" \
  "先修好编不过或红着的单测（kb 里的实测结论全靠它们背书）；改完只重跑这一格：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests"
stage_cell wallclock-asserts-registered cell_wallclock_asserts_registered "实验单测里按挂钟判的断言都有登记位（一行注释），没登记的判红" \
  "按判红点名的文件:行号，在 #[test] 与 fn 之间写一行 // wallclock-assert: <量的是什么>（只写注释，装置代码不动）；负载再高也不会红的写 // wallclock-assert-load-safe: <为什么>；登记名撞上别的测试名的，改名或把那一条也登记；改完只重跑这一格：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check wallclock-asserts-registered"
stage_cell experiment-replay cell_experiment_replay "入库的纯模型实验重跑一遍，与 research/results/ 里的产物逐字节相同" \
  "对不上先判是代码坏了还是产物该更新（两种都要人判）；只重跑红的那几个实验：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check experiment-replay --item <实验号>"
stage_cell mutation-tables-compile cell_mutation_tables_compile "研究侧每张变异表逐条施加替换文只编译，一条编不过就红" \
  "改那一行替换文（编得过、仍改被测行为），或装置改回能编；表归实验执行员改。单张核：在 scripts/experiment-apparatus/ 里 bash ../mutation/mutate.sh --compile-only <bin> <源文件> <表>；改完只重跑这一格：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check mutation-tables-compile"
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
R=scripts/experiment-apparatus
REPLAY_SCRIPT="scripts/experiment-apparatus/replay.sh"
ADMISSION="$STAGE_REPOSITORY/scripts/gate/admission.py"

# replay.sh 表里的行：编号|二进制|参数|产物|exact 或 timing。门禁靠这个形状取行，改表格式要同时改这里。
replay_rows_of_kind() { sed -n "s/^\(E[0-9]*\)|[^|]*|[^|]*|[^|]*|$1\$/\1/p" "$REPLAY_SCRIPT" 2>/dev/null; }
SLOW=(E9 E17 E20 E21)

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
    slow_note=""
    for s in "${SLOW[@]}"; do [[ "$s" == "$experiment" ]] && slow_note="（慢，默认不跑）"; done
    [[ "$kinds" == timing ]] && slow_note="（只有 timing 行，默认不跑）"
    printf '%s\t%s%s\n' "$experiment" "$kinds" "$slow_note"
    listed=$((listed + 1))
  done < <({ replay_rows_of_kind exact; replay_rows_of_kind timing; } | awk '!seen[$0]++')
  ((listed)) || { echo "  ✗ 从 $REPLAY_SCRIPT 里一行都没读到：表的格式变了？"; echo "     → 怎么办：这里靠 '编号|二进制|参数|产物|exact 或 timing' 这个形状取行，改表格式要同时改这里。"; exit 1; }
  # 末尾一行：复跑表每一行的行键对不对得上按行判定库（admission.py replay-row-inputs 只算键，不编不跑；GPU 开关照没开算，开了的行键另算）
  list_library="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)"
  if [[ -n "$list_library" ]] && list_keys="$(python3 "$ADMISSION" replay-row-inputs . --gpu 0 2>/dev/null)"; then
    list_library="$list_library/singlefs-unit-verdicts/replay-row"
    list_same=0; list_mismatch=0; list_refused=0
    while IFS=$'\t' read -r _list_identity list_key _list_units _list_note; do
      [[ -n "${list_key:-}" ]] || continue
      if [[ "$list_key" == refused ]]; then list_refused=$((list_refused + 1))
      elif [[ -f "$list_library/keys/$list_key/verdict" ]]; then list_same=$((list_same + 1))
      else list_mismatch=$((list_mismatch + 1)); fi
    done <<<"$list_keys"
    printf '复跑表行键\t键对不上 %s 行、键相同 %s 行、拒算 %s 行（照 GPU 开关没开算；库 %s）\n' "$list_mismatch" "$list_same" "$list_refused" "$list_library"
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

# research_unit_tests_plan <草稿目录>：每个包跑一次 admission.py test-plan（文件头 research-unit-tests 那一段「按条复用」），计划输出并进 <草稿目录>/plan.out、
# 各包的计划文件放 <草稿目录>/plan.<包>；四个数放进 unit_plan_judge、unit_plan_reuse、unit_plan_refused；算不出返回 1、原因放进 unit_plan_problem。
# 样本（被判的根不是这道阶段所在的仓）另放了 .unit-verdict-sample/prior/ 的，先拿上一版算键、照判绿记进这一版样本的判定库。
# 弄坏开关 unit-tests-without-library 下带 --full（只写不读：一条都不复用）。
research_unit_tests_plan() {
  local scratch="$1" package summary seeded plan_exit
  local -a tokens=(--root "$R" --profile release --token "judge:scripts/gate/lib_libtest_result_lines.py::result_line" --token "judge-root:$STAGE_REPOSITORY")
  local -a full_option=()
  research_build_broken unit-tests-without-library && full_option=(--full)
  unit_plan_judge=0; unit_plan_reuse=0; unit_plan_refused=0; unit_plan_problem=""; unit_plan_failure=()
  : >"$scratch/plan.out"
  mapfile -t unit_plan_packages < <(research_test_packages)
  if ((${#unit_plan_packages[@]} == 0)); then unit_plan_problem="$R/Cargo.toml 里读不出包名"; return 1; fi
  for package in "${unit_plan_packages[@]}"; do
    if [[ "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" && -d .unit-verdict-sample/prior ]]; then
      if python3 "$ADMISSION" test-plan "$ROOT/.unit-verdict-sample/prior" "$STAGE_NAME:research-unit-tests" "$package" "$scratch/seed.$package" "${tokens[@]}" \
           --library-root "$ROOT" >"$scratch/seed.$package.out" 2>&1; then plan_exit=0; else plan_exit=$?; fi
      if ((plan_exit != 0)); then
        unit_plan_problem="样本：上一版的树上算不出键（$(tail -n 2 "$scratch/seed.$package.out" | tr '\n' ' ' | cut -c1-300)）"
        unit_plan_failure=(test-plan "$plan_exit" "$scratch/seed.$package.out"); return 1
      fi
      awk -F'\t' 'NF >= 7 && ($2 == "judge" || $2 == "reuse") { print $1 }' "$scratch/seed.$package.out" >"$scratch/seed.$package.names"
      seeded="$(python3 "$ADMISSION" verdict-record "$ROOT/.unit-verdict-sample/prior" "$STAGE_NAME:research-unit-tests" "$scratch/seed.$package" "$scratch/seed.$package.names" \
        --how "样本：上一版" --library-root "$ROOT" 2>&1 | tail -n 1)"
      echo "  · 样本：上一版（.unit-verdict-sample/prior）$package 的 $(grep -c . "$scratch/seed.$package.names") 条测试函数照判绿记进这一版的判定库（$seeded）"
    fi
    if python3 "$ADMISSION" test-plan "$ROOT" "$STAGE_NAME:research-unit-tests" "$package" "$scratch/plan.$package" "${tokens[@]}" ${full_option[@]+"${full_option[@]}"} \
         >"$scratch/plan.$package.out" 2>&1; then plan_exit=0; else plan_exit=$?; fi
    if ((plan_exit != 0)); then
      unit_plan_problem="admission.py test-plan $package 没成（$(tail -n 2 "$scratch/plan.$package.out" | tr '\n' ' ' | cut -c1-300)）"
      unit_plan_failure=(test-plan "$plan_exit" "$scratch/plan.$package.out"); return 1
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
# 失灵它打 ✗ 与出路，这里记 unit_plan_red=1，这一格照旧跑完、结论判红
research_unit_plan_failure_verdict() {
  python3 "$ADMISSION" plan-failure "$STAGE_NAME 的 research-unit-tests 格" "$@" || unit_plan_red=1
}

cell_research_unit_tests() {
  local out rc b n lint listing listing_rc name joined scratch plan_ok=1 output_file wallclock_rows
  local registered_wallclock=() skipped_wallclock=() ran_wallclock=() skip_arguments=() wallclock_exact=()
  local MEMORY_CAP_RUNNER RESEARCH_BUILD_MEMORY_MAX
  [[ -f "$R/Cargo.toml" ]] || { echo "  ! 没有 $R/Cargo.toml，这一格无对象可判"; exit 77; }
  command -v cargo >/dev/null || {
    echo "  ✗ 没有 cargo，装不了就没法验 research 的证据"
    echo "     → 怎么办：装 Rust 工具链（curl https://sh.rustup.rs -sSf | sh），"
    echo "               或把已装的 cargo 放进 PATH。跳过这一步等于 research/ 的数字没人验过。"
    exit 1; }
  MEMORY_CAP_RUNNER="$STAGE_REPOSITORY/scripts/hosts/run-with-memory-cap.sh"
  RESEARCH_BUILD_MEMORY_MAX="${GATE_RESEARCH_BUILD_MEMORY_MAX:-8G}"
  if [[ ! "$RESEARCH_BUILD_MEMORY_MAX" =~ ^[1-9][0-9]*[KMGT]$ ]]; then
    echo "  ✗ GATE_RESEARCH_BUILD_MEMORY_MAX 写成了「$RESEARCH_BUILD_MEMORY_MAX」，内存包装不认（或认成字节数）：这一格没跑"
    echo "     → 怎么办：写成正整数加 K / M / G / T（例 16G），单位必写；不设就用默认 8G"
    exit 1
  fi
  scratch="$(mktemp -d)" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  # shellcheck disable=SC2064  # 路径在这里就展开：退出时 local 变量已经不在
  trap "rm -rf -- $(printf '%q' "$scratch")" EXIT
  # 先编整个 research 工作区的 release 产物：复跑那一格的 replay.sh 编的就是这一份，跑到那一格时 cargo 判它新鲜、不再编。
  # 「编得过」判的是整个目标（闭包够不到的死码在 deny 警告下也编不过），单元闭包罩不住，这一步每趟照编（增量编，没改动时 cargo 判新鲜）。
  # cargo 的编译与测试线程不吃 cargo 自己的缺省（核数）：经 capped.sh default 起，取登记的重活默认并行度（.claude/heavy-thread-default，SINGLEFS_THREAD_CAP 设了取它）
  out="$(cd "$R" && bash "$STAGE_REPOSITORY/scripts/hosts/capped.sh" default cargo build --release 2>&1)"
  rc=$?
  if (( rc != 0 )); then
    grep -E '^error' <<<"$out" | head -8 | sed 's/^/     /'
    echo "  ✗ research 编不过（cargo build --release 退出码 $rc）"
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
  unit_plan_red=0
  if research_unit_tests_plan "$scratch"; then
    echo "  · 单测按条复用：$((unit_plan_judge + unit_plan_reuse + unit_plan_refused)) 条编得进的测试函数，现判 ${unit_plan_judge}、复用 ${unit_plan_reuse}、拒算 ${unit_plan_refused}"
    awk -F'\t' 'NF >= 8 && $3 == "judge" { if (judged++ < 20) printf "    现判 %s（%s）\n", $2, substr($8, 1, 300) }
                NF >= 8 && $3 == "refused" { if (refused++ < 20) printf "    拒算 %s（%s）\n", $2, substr($8, 1, 300) }
                END { if (judged > 20) printf "    …现判另有 %d 条没列\n", judged - 20; if (refused > 20) printf "    …拒算另有 %d 条没列\n", refused - 20 }' "$scratch/plan.out"
  else
    plan_ok=0
    echo "  ! 单测按条复用算不出（$unit_plan_problem）：这一趟全部现判、一条都不记库"
    ((${#unit_plan_failure[@]})) && research_unit_plan_failure_verdict "${unit_plan_failure[@]}"
  fi
  # 登记了的挂钟断言按名跳过；弄坏开关 skip-filter-dropped（文件头）：不传，它们照跑。
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
    if (( unit_plan_judge + unit_plan_refused == 0 )) || (( unit_plan_judge + unit_plan_refused == $(grep -c . "$scratch/wallclock.names") )); then
      for name in ${registered_wallclock[@]+"${registered_wallclock[@]}"}; do
        echo "  ! 本次未跑：挂钟断言 $name 本次未跑（cargo test --skip：门禁自己的并发会让它误红，要验在不并发的场合单独跑 cargo test）"
        if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$STAGE_NAME 的 research-unit-tests 格：挂钟断言 $name 本次未跑" >> "$GATE_NOT_RUN_FILE"; fi
      done
      echo "  ✓ research 构建与单测全绿：单测 ${unit_plan_reuse} 条都有键相同的判定，全部复用，这一趟没起 cargo test（现判 0、复用 ${unit_plan_reuse}、拒算 0、本次未跑 ${#registered_wallclock[@]}）"
      exit 0
    fi
  fi
  output_file="$scratch/cargo-test.out"
  (cd "$R" && RUN_WITH_MEMORY_CAP_KEY="gate checker-tier-research-build-and-replay: cargo test --release (research)" \
    bash "$MEMORY_CAP_RUNNER" "$RESEARCH_BUILD_MEMORY_MAX" bash "$STAGE_REPOSITORY/scripts/hosts/capped.sh" default cargo test --release ${skip_arguments[@]+"${skip_arguments[@]}"}) >"$output_file" 2>&1
  rc=$?
  out="$(cat "$output_file")"
  if (( rc == 245 || (rc >= 250 && rc <= 254) )); then
    tail -8 <<<"$out" | sed 's/^/     /'
    echo "  ✗ research 的构建与单测没判出结果：内存包装 scripts/hosts/run-with-memory-cap.sh 退 $rc（上限 $RESEARCH_BUILD_MEMORY_MAX），这一次的输出不算判定"
    echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一格要的比上限多，把 GATE_RESEARCH_BUILD_MEMORY_MAX 设大再跑；"
    echo "               251、252、254 先跑 bash scripts/hosts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格；不拿掉包装跑。"
    echo "               253 是超过限时：把 RUN_WITH_MEMORY_CAP_TIME_LIMIT 拿掉或放宽再跑。245 是同一段时间内核报了显存不够：nvidia-smi 看谁在用卡，等它跑完再重跑这一格。"
    exit 1
  fi
  if (( rc != 0 )); then
    echo "  ✗ research 的构建或单测没过（cargo test 退出码 $rc）"
    grep -E '^error|FAILED|panicked at' <<<"$out" | head -8 | sed 's/^/     /'
    if ((plan_ok)); then
      while IFS= read -r name; do
        awk -F'\t' -v name="$name" 'NF >= 8 && $7 == name && ($3 == "judge" || $3 == "refused") { printf "     红的 %s：%s\n", $2, substr($8, 1, 400) }' "$scratch/plan.out"
      done < <(sed -n 's/^test \(.*\) \.\.\. FAILED$/\1/p' <<<"$out" | LC_ALL=C sort -u)
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
  # 登记了的按挂钟判的断言：--skip 之后输出里不该再有它们的 test … ok / FAILED 行（被 --skip 滤掉的测试不打任何一行）
  for name in ${registered_wallclock[@]+"${registered_wallclock[@]}"}; do
    if grep -Eq -- "^test (.*::)?${name} \\.\\.\\. " <<<"$out"; then ran_wallclock+=("$name"); else skipped_wallclock+=("$name"); fi
  done
  if ((${#ran_wallclock[@]} > 0)); then
    echo "  ✗ 登记了的挂钟断言 ${#ran_wallclock[@]} 条（${ran_wallclock[*]}）在 cargo test 的输出里以 test … 行出现：登记了却跑了，--skip 没生效，它们在门禁的负载下照跑了"
    echo "     → 怎么办：看起 cargo test 的命令里有没有 -- --skip <函数名>（判法 python3 scripts/gate/wallclock-assert-lint.py --list . 列登记名）、"
    echo "               登记名是不是被别的写法拼错；--skip 是子串匹配，函数名要和登记的一字不差。"
    exit 1
  fi
  # 研究单测读不到别的实验的产物、拿来当期望值的留存产物时打的本次未跑行（scripts/experiment-apparatus/lib.rs 的 ARCHIVED_PRODUCT_NOT_RUN_MARKER：
  # 「NOT_RUN_ARCHIVED_PRODUCT test=<目标根文件名去掉 .rs>::<libtest 全名> reason=<一句>」）：那几条 libtest 照判 ok，这里逐条报本次未跑、不记库；
  # 按条复用算得出时，test= 对不上计划里任何一条判红（名字写错的那一条会被当成判过的绿记进库）。弄坏开关 archived-not-run-ignored：不认这一种行（改前的样子）
  local archived_not_run=() archived_unmatched=() marker_test marker_reason plan_name
  if ! research_build_broken archived-not-run-ignored; then
    while IFS=$'\t' read -r marker_test marker_reason; do
      [[ -n "$marker_test" ]] || continue
      archived_not_run+=("$marker_test")
      if ((plan_ok)); then
        plan_name="$(awk -F'\t' -v name="$marker_test" 'NF >= 8 { root = $6; sub(/^.*\//, "", root); sub(/\.rs$/, "", root); if (root "::" $7 == name) print $2 }' "$scratch/plan.out")"
        if [[ -n "$plan_name" ]]; then printf '%s\n' "$plan_name" >>"$scratch/wallclock.names"; else archived_unmatched+=("$marker_test"); fi
      fi
      echo "  ! 本次未跑：$marker_test（$marker_reason）"
      if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$STAGE_NAME 的 research-unit-tests 格：$marker_test 读不到留存产物，本次未跑" >> "$GATE_NOT_RUN_FILE"; fi
    done < <(archived_product_not_run_lines "$output_file")
  fi
  if ((${#archived_unmatched[@]})); then
    echo "  ✗ 本次未跑行点名的 ${#archived_unmatched[@]} 条用例（${archived_unmatched[*]}）在按条复用的计划里找不到：名字写错的那一条会被当成判过的绿记进库"
    echo "     → 怎么办：调用处给的用例名写成 <目标根文件名去掉 .rs>::<libtest 全名>（例 e134_map_key_slot_baselines::tests::<函数名>，写法见 scripts/experiment-apparatus/lib.rs 的 ARCHIVED_PRODUCT_NOT_RUN_MARKER），"
    echo "               改完只跑这一格：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests"
    exit 1
  fi
  ((plan_ok)) && research_unit_tests_record "$scratch" "$output_file" "$scratch/wallclock.names"
  if ((unit_plan_red)); then
    echo "  ✗ research-unit-tests 这一格判红：单测按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；这一趟照旧全部现判跑完、$b 个测试批次共 $n 个单测都过了"
    echo "     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再只跑这一格：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check research-unit-tests；不能复用就判红，不退回全部现判当绿"
    exit 1
  fi
  joined=""
  for name in ${skipped_wallclock[@]+"${skipped_wallclock[@]}"}; do
    echo "  ! 本次未跑：挂钟断言 $name 本次未跑（cargo test --skip：门禁自己的并发会让它误红，要验在不并发的场合单独跑 cargo test）"
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$STAGE_NAME 的 research-unit-tests 格：挂钟断言 $name 本次未跑" >> "$GATE_NOT_RUN_FILE"; fi
    joined+="${joined:+、}$name"
  done
  local not_run_total=$(( ${#skipped_wallclock[@]} + ${#archived_not_run[@]} ))
  local unit_counts="单测按条：现判 ${unit_plan_judge}、复用 ${unit_plan_reuse}、拒算 ${unit_plan_refused}、本次未跑 ${not_run_total}"
  ((plan_ok)) || unit_counts="单测按条复用算不出，全部现判、本次未跑 ${not_run_total}"
  ((${#archived_not_run[@]})) && unit_counts+="；读不到留存产物本次未跑 ${#archived_not_run[@]} 条（上面逐条列名），它们不记为绿"
  if ((${#skipped_wallclock[@]})); then
    echo "  ✓ research 构建与单测全绿：$b 个测试批次、共 $n 个单测（${unit_counts}）；跳过挂钟断言 ${#skipped_wallclock[@]} 条（$joined），它们本次未跑，不记为绿"
  else
    echo "  ✓ research 构建与单测全绿：$b 个测试批次、共 $n 个单测（${unit_counts}）"
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
  local replay_rc replay_log replay_total judged reused refused archived_run_only load_skipped inputs_not_run
  local -a break_setting=()
  # 弄坏开关 row-key-with-product-name（文件头）：交给 replay.sh 起的 admission.py（ADMISSION_BREAK），行键另加产物文件名
  research_build_broken row-key-with-product-name && break_setting=(ADMISSION_BREAK=row-key-with-product-name)
  # RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：replay.sh 编 checker 档的装置与 E162 那几个 bin（都编到 librocksdb-sys），
  #   起它之前判一次这台装了、此刻进程环境的取值对得上，判红这一格退 1、replay.sh 不起（一个 cargo 都不起）。树里没有依赖 rocksdb 的清单（别的样本）、
  #   PATH 上是带标记的假 cargo，entry 报不判。弄坏开关 rocksdb-entry-unchecked 下不判（样本 experiment-replay-rocksdb-entry-refused 判错）
  #   这一份被拷到不带 scripts/hosts/ 的树里时报一行没判、往下走
  if ! research_build_broken rocksdb-entry-unchecked && [[ ! -f "$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py" ]]; then
    echo "  ! RocksDB 预编库的取值没判：$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py 不在（这一份阶段被拷到了不带 scripts/hosts/ 的树里）"
  elif ! research_build_broken rocksdb-entry-unchecked && ! python3 "$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py" entry --root "$ROOT"; then
    echo "  ✗ 复跑：起 replay.sh（它编到 librocksdb-sys）之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一个实验都没跑"
    echo "    → 怎么办：照上面的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"），再只跑这一格：bash .claude/gate.d/checker-tier-research-build-and-replay.sh --check experiment-replay"
    exit 1
  fi
  replay_log="$(mktemp)"
  env ${break_setting[@]+"${break_setting[@]}"} bash "$REPLAY_SCRIPT" "$@" | tee "$replay_log"
  replay_rc="${PIPESTATUS[0]}"
  # 按行复用（replay.sh 文件头「按行复用」）：照它末尾那一行 REPLAY_TOTAL 报现判、复用与已归档只跑不比的行数；没有那一行报取不出，不写 0
  replay_total="$(grep '^REPLAY_TOTAL ' "$replay_log" | tail -n 1)"
  rm -f -- "${replay_log:?}"
  if [[ -n "$replay_total" ]] && ! research_build_broken replay-reuse-unreported; then
    judged="$(sed -n 's/.* judged=\([0-9]*\).*/\1/p' <<<"$replay_total")"
    reused="$(sed -n 's/.* reused=\([0-9]*\).*/\1/p' <<<"$replay_total")"
    refused="$(sed -n 's/.* refused=\([0-9]*\).*/\1/p' <<<"$replay_total")"
    archived_run_only="$(sed -n 's/.* archived_run_only=\([0-9]*\).*/\1/p' <<<"$replay_total")"
    load_skipped="$(sed -n 's/.* load_skipped=\([0-9]*\).*/\1/p' <<<"$replay_total")"
    # not_run=：装置或驱动读不到别的实验的产物、当期望值的留存产物，那一行记本次未跑（replay.sh 文件头「本次未跑」）；弄坏开关 replay-not-run-uncounted 不读它（改前的样子）
    inputs_not_run=""
    research_build_broken replay-not-run-uncounted || inputs_not_run="$(sed -n 's/.* not_run=\([0-9]*\).*/\1/p' <<<"$replay_total")"
    echo "  · 复跑：现判 ${judged:-取不出} 行、复用 ${reused:-取不出} 行、拒算 ${refused:-取不出} 行、本次未跑 $(( ${archived_run_only:-0} + ${load_skipped:-0} + ${inputs_not_run:-0} )) 行（已归档只跑不比 ${archived_run_only:-取不出} 行、负载高没判 ${load_skipped:-取不出} 行、输入不在 ${inputs_not_run:-取不出} 行；replay.sh 按行复用：键相同、判过的那一行不跑，见它文件头「按行复用」）"
  elif ! research_build_broken replay-reuse-unreported; then
    echo "  ! 复跑：复用数取不出（replay.sh 没打 REPLAY_TOTAL 行），这一趟现判几行、复用几行这一格报不出"
  fi
  case "$replay_rc" in
    0) return 0 ;;
    77)
      # replay.sh 退 77：整趟没有一行判得了，判的全是负载高本次不判的行与输入不在本次未跑的行（它文件头「本次未跑」末句）；照 REPLAY_TOTAL 说是哪一种
      local why_not_judged="负载高"
      if [[ "${inputs_not_run:-0}" != 0 && "${load_skipped:-0}" == 0 ]]; then why_not_judged="要读的产物全不在"
      elif [[ "${inputs_not_run:-0}" != 0 ]]; then why_not_judged="负载高 ${load_skipped} 行、要读的产物不在 ${inputs_not_run} 行"
      fi
      echo "  ! 复跑整趟本次不判（${why_not_judged}）：replay.sh 退 77，一行都没比；不算通过也不算红"
      if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$STAGE_NAME 的 experiment-replay 格：复跑整趟本次不判（${why_not_judged}）" >> "$GATE_NOT_RUN_FILE"; fi
      exit 77 ;;
    *) exit 1 ;;
  esac
}

cell_experiment_replay() {
  local e s skip dup ALL_EXACT=() ALL_TIMING=() FAST=() SKIPPED=()
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
  mapfile -t ALL_TIMING < <(replay_rows_of_kind timing)
  for e in "${ALL_EXACT[@]}"; do
    skip=0
    for s in "${SLOW[@]}"; do [[ "$e" == "$s" ]] && skip=1; done
    if ((skip)); then SKIPPED+=("$e（慢）"); else FAST+=("$e"); fi
  done
  # timing 行默认一行都不跑：逐个进「没跑」清单。去重只比已经进了清单的那几行（同一个编号既有 exact 行又有 timing 行时），
  # 不比 SLOW：SLOW 里的编号若只有 timing 行，拿 SLOW 去重会让它既不跑、也不进清单。
  for e in ${ALL_TIMING[@]+"${ALL_TIMING[@]}"}; do
    dup=0
    for s in ${SKIPPED[@]+"${SKIPPED[@]}"}; do [[ "$s" == "$e（慢）" || "$s" == "$e（计时）" ]] && dup=1; done
    ((dup)) || SKIPPED+=("$e（计时）")
  done

  if [[ "${GATE_REPLAY_FULL:-0}" == 1 ]]; then
    run_replay
    exit 0
  fi

  run_replay "${FAST[@]}"
  # 派给 replay.sh 的行里，登记了准入（.claude/gate.d/stage-inputs.tsv）而输入自留存产物以来没变的不跑，
  # replay.sh 汇总行报「输入没变没跑 N」并逐个列名；这里不说成「跑了」。
  echo "  ! 本次派给 replay.sh ${#FAST[@]} 个（表里的 exact 行减掉慢的；其中准入判「输入没变」而没跑的，见上面 replay.sh 汇总里那一行）；没派 ${#SKIPPED[@]} 个：${SKIPPED[*]}"
  echo "    标（慢）的是 SLOW 清单（${SLOW[*]}）里判定为 exact 的行；标（计时）的是表里判定为 timing 的行，默认档一行都不跑。"
  echo "    跑： GATE_REPLAY_FULL=1 bash .claude/scripts/gate.sh，或逐个点名：bash .claude/gate.d/${STAGE_NAME} --check experiment-replay --item <实验号>"
  echo "    ⚠️ 它们的结论靠 replay.sh 里的区间断言钉住，不跑就等于那几条断言本轮没验。"
}

# ── mutation-tables-compile ──
research_build_broken() { [[ ",${RESEARCH_BUILD_AND_REPLAY_BREAK:-}," == *",$1,"* ]]; }

# archived_product_not_run_lines <cargo test 的输出>：每一处 NOT_RUN_ARCHIVED_PRODUCT（单测直接写标准输出，不带 --nocapture 也在；
# 可能跟在「test <名> ... 」后面，按子串找），一处一行「<test= 的值><TAB><reason= 之后的整句>」
archived_product_not_run_lines() {
  awk '{
    rest = $0
    while ((at = index(rest, "NOT_RUN_ARCHIVED_PRODUCT test=")) > 0) {
      rest = substr(rest, at + length("NOT_RUN_ARCHIVED_PRODUCT test="))
      space = index(rest, " ")
      name = (space > 0) ? substr(rest, 1, space - 1) : rest
      reason = ""
      if (match(rest, / reason=/)) reason = substr(rest, RSTART + 8)
      following = index(reason, "NOT_RUN_ARCHIVED_PRODUCT ")
      if (following > 0) reason = substr(reason, 1, following - 1)
      sub(/[[:space:]]+$/, "", reason)
      print name "\t" reason
    }
  }' "$1"
}

# mutation_counts_of_log <mutate.sh 的输出>：它末尾「计数：」那一行里的现判、复用、拒算条数，打成「<现判> <复用> <拒算>」；取不出打空串
mutation_counts_of_log() {
  sed -n 's/^计数：.*、现判 \([0-9]*\) 条、复用 \([0-9]*\) 条、拒算 \([0-9]*\) 条$/\1 \2 \3/p' "$1" | tail -n 1
}

cell_mutation_tables_compile() {
  local tables_directory=scripts/mutation/tables mutate_script="$STAGE_REPOSITORY/scripts/mutation/mutate.sh"
  local sample_mode=0 rounds=1 round library="" target_directory scratch cleanup_paths="" library_root_for_break=""
  local table stem binary rc log rows line counts fresh reused refused table_library first_lines
  local fresh_total reused_total refused_total tables_judged tables_all_reused compiled_rows invalid_total
  local -a table_library=()
  local -a not_run=() red_lines=() red_tables=() invalid_lines=() broken_environment=() full_option=() library_setting=()
  [[ -d "$R" && -d "$tables_directory" ]] || { echo "  ! 没有 $R 或 $tables_directory，这一格无对象可判"; exit 77; }
  if ! command -v cargo >/dev/null; then
    echo "  ✗ 没有 cargo：变异表的替换文编不编得过判不了"
    echo "     → 怎么办：装 Rust 工具链或把已装的 cargo 放进 PATH，再只重跑这一格：bash .claude/gate.d/${STAGE_NAME} --check mutation-tables-compile"
    exit 1
  fi
  if [[ ! -f "$mutate_script" ]]; then
    echo "  ✗ 没有 $mutate_script：这一格的判法（--compile-only、--binary-of）与按条判定库都在它里面"
    echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一格按红记。"
    exit 1
  fi
  scratch="$(mktemp -d)" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  cleanup_paths="$(printf '%q' "$scratch")"
  # 编译目录一律用这一格自己的临时目录，跑完删；样本（被判的根不是这道阶段所在的仓、也不是 --staged）另把按条判定库也放这里；
  # 只有 run-twice 那份接连判两趟（同一份库，第二趟照条复用），另放了 second-round/ 的在第二趟之前把它下面的文件盖到样本根上（改一条替换文）
  if [[ "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" ]]; then sample_mode=1; fi
  target_directory="$scratch/target"   # 这一格自己的编译目录（文件头 mutation-tables-compile），收尾随 $scratch 删
  if ((sample_mode != 0)); then
    library="$scratch/library"
    library_setting=("MUTATE_LIBRARY_DIRECTORY=$library")
    [[ -f .mutation-tables-compile-sample/run-twice ]] && rounds=2
  fi
  # shellcheck disable=SC2064  # 路径在这里就展开（%q 过）：退出时 local 变量已经不在
  trap "rm -rf -- $cleanup_paths" EXIT
  # SINGLEFS_GATE_FULL=1：mutate.sh 带 --full（只写不读，每条现判、照键写回同一份库）
  [[ "${SINGLEFS_GATE_FULL:-}" == 1 ]] && full_option=(--full)
  # 弄坏开关 no-cap-lints（文件头）：PATH 前面放垫片 cargo，剥掉 RUSTFLAGS 与 CARGO_ENCODED_RUSTFLAGS 再交给真 cargo，mutate.sh 带的 cap 一条都到不了 rustc
  if research_build_broken no-cap-lints; then
    local real_cargo
    real_cargo="$(command -v cargo)"
    mkdir "$scratch/no-cap-bin" && printf '#!/usr/bin/env bash\nexec env -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS %q "$@"\n' "$real_cargo" >"$scratch/no-cap-bin/cargo" \
      && chmod +x "$scratch/no-cap-bin/cargo" || { echo "  ✗ 弄坏开关 no-cap-lints 建不了垫片 cargo"; echo "     → 怎么办：看自证临时目录能不能写、PATH 上有没有 cargo，修好再跑自证"; exit 1; }
    broken_environment=("PATH=$scratch/no-cap-bin:$PATH")
  fi
  # 弄坏开关 mutation-compile-key-whole-table（文件头）：按条判定库按整张表的 sha256 分开放（改前那种按表一项的样子：表里改一条，整张表的条都作废）
  if research_build_broken mutation-compile-key-whole-table; then
    library_root_for_break="${library:-$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)/singlefs-mutate-library-by-table}"
  fi
  # E162 的 rocksdb 要 BINDGEN_EXTRA_CLANG_ARGS（mutate.sh 文件头 required-features 那一段：由调用方设）；没设就取放量共用库里那一份
  if [[ -z "${BINDGEN_EXTRA_CLANG_ARGS:-}" && -f "$STAGE_REPOSITORY/scripts/crash/crash-amplification-lib.sh" ]]; then
    source "$STAGE_REPOSITORY/scripts/crash/crash-amplification-lib.sh"
    BINDGEN_EXTRA_CLANG_ARGS="${CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS:-}"
    export BINDGEN_EXTRA_CLANG_ARGS
  fi
  for ((round = 1; round <= rounds; round++)); do
    if ((rounds > 1)); then
      if ((round == 2)) && [[ -d .mutation-tables-compile-sample/second-round ]]; then
        cp -a .mutation-tables-compile-sample/second-round/. . || { echo "  ✗ 样本：第二趟之前盖不上 .mutation-tables-compile-sample/second-round/"; echo "     → 怎么办：核样本目录里的 second-round/ 在不在、能不能读，补齐样本再跑"; exit 1; }
        echo "  ── 第 2 趟之前把 .mutation-tables-compile-sample/second-round/ 下的文件盖到样本根上（改了表里一条）"
      fi
      echo "  ── 第 $round 趟（样本 .mutation-tables-compile-sample/run-twice：按条判定库 $library，第二趟没改的条应当复用）"
    fi
    not_run=(); red_lines=(); red_tables=(); fresh_total=0; reused_total=0; refused_total=0; tables_judged=0; tables_all_reused=0; compiled_rows=0; invalid_total=0
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
            bash "$mutate_script" ${full_option[@]+"${full_option[@]}"} --compile-only "$binary" "$stem.rs" "../mutation/tables/$stem.tsv") >"$log" 2>&1; then rc=0; else rc=$?; fi
      tables_judged=$((tables_judged + 1))
      research_build_broken invalid-rows-ignored && ((rc == 1)) && rc=0   # 弄坏开关：⏭ 不看，退 1 的表也当绿
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
        0)
          rows="$(grep -c '^✅ ' "$log")"
          compiled_rows=$((compiled_rows + rows))
          echo "  · $table：$rows 条全编得过（$counts）"
          ;;
        1)
          mapfile -t invalid_lines < <(grep '^⏭ ' "$log")
          if ((${#invalid_lines[@]})); then
            for line in "${invalid_lines[@]}"; do
              red_lines+=("$table：${line#⏭  }")
              # 这一条这一趟为什么现判（mutate.sh 在 stderr 逐条写「现判：[<名>] <原因>」，键对不上时原因里是变了的单元）
              first_lines="$(grep -F "mutate: 现判：[$(sed -n 's/^⏭  \[\([^]]*\)\].*/\1/p' <<<"$line")]" "$log" | head -n 1 | sed 's/^mutate: 现判：//')"
              [[ -n "$first_lines" ]] && red_lines+=("    这一条为什么现判：${first_lines}")
            done
            invalid_total=$((invalid_total + ${#invalid_lines[@]}))
            echo "  · $table：${#invalid_lines[@]} 条编不过（$counts）"
          else
            red_tables+=("$table：mutate.sh --compile-only 退 1 而一行 ⏭ 都没有；输出末尾：$(tail -n 3 "$log" | tr '\n' ' ')")
          fi
          ;;
        *)
          red_tables+=("$table：mutate.sh --compile-only $binary $stem.rs 退 $rc（2 基线编不过或用法错、3 锚点没命中、5 跑的时候源文件被改、6 bin 与源文件对不上、8 收回的判定对不上）；输出末尾：$(tail -n 4 "$log" | tr '\n' ' ')")
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
      echo "  ✗ 研究侧变异表有 $invalid_total 条替换文编不过、${#red_tables[@]} 张表没判完（按条：现判 $fresh_total 条、复用 $reused_total 条、拒算 $refused_total 条）"
      echo "     → 怎么办：编不过的改那一行替换文（编得过、仍改被测行为），或装置改回能编；没判完的照括号里 mutate.sh 的退出码办。"
      echo "               单张核：(cd scripts/experiment-apparatus && bash ../mutation/mutate.sh --compile-only <bin> <源文件> ../mutation/tables/<表>)"
      exit 1
    fi
  done
  for line in ${not_run[@]+"${not_run[@]}"}; do
    echo "  ! 本次未跑：$line"
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$STAGE_NAME 的 mutation-tables-compile 格没判 $line" >> "$GATE_NOT_RUN_FILE"; fi
  done
  if ((tables_judged == 0)); then
    echo "  ! 本次无对象可判：$tables_directory 下没有一张表对得上 $R/ 下同名的 .rs（没跑 ${#not_run[@]} 张，上面逐张列名）"
    exit 77
  fi
  echo "  ✓ 研究侧变异表逐条只编译全编得过：判了 $tables_judged 张表（整张复用、一条都没编的 $tables_all_reused 张），按条：现判 $fresh_total、复用 $reused_total、拒算 $refused_total、本次未跑 ${#not_run[@]} 张表（上面「本次未跑」逐张列名）"
}

stage_cells_run "$ROOT"
