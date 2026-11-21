#!/usr/bin/env python3
# admission: always 判的是此刻被判的仓、它的 crates/mutations.tsv 与 git common-dir 里那一份判定库，上一次的结论不替这一次作保
# run-condition: none 算行键要的 git 由 admission.py 的 git_common_directory 起、起不来报错退 2（行键不起 cargo 与 rustc）；llvm-profdata、llvm-cov 由 run_stage 现找、找不到判红；--selftest 用自己造的假 cargo 与假 llvm 工具，不编译
"""门禁 crates-mutation-replay（crates 变异表复跑）的判法与判定库，连同门禁 code-source-discipline 静态判的那几样：一份判法只在这里，两道阶段都调它。

变异表一行成形 = 六段制表符分隔（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名）；替换文可以空（删掉原文：改坏时原文换成空串），
其余五段一段都不许空（malformed_reason，read_table 照它读；crates-mutation-replay、static-check 同一份）。

crates-mutation-replay 经 import 调 run_stage（阶段文件里那几行内嵌 python）；这里不给整张表开命令行入口：项目代码闸（.claude/hooks/project-code-guard.sh）
按阶段文件名认 crates-mutation-replay，另开一个入口它认不出。

拆成的几份（本文件只留文件头、子命令分派 main 与对各份的再导出；代码在同目录的这几份里，按这个次序往前 import、没有环；别处按本文件取名字，名字一个不少）：
  crates_mutation_rows_base.py  标准库与仓内库的载入、全部模块级赋值（常量、表、正则、假 cargo 与假 llvm 工具的正文、模块级可变状态如 WORKER_STATE 与 RUN_BUILD_REMOVAL_FAILURES：别的份 import 同一个对象，不各存一份）、弄坏开关的读取（break_is）、say / note、变异表一行的成形与读表、拷贝范围的头一段，以及几份共用的小函数。模块级赋值都在这一份（自证用的两张模块表不在这一份：FIXTURE_MODULES 在 driver、WRAPPED_FUNCTION_MODULES 在 commands）。
  crates_mutation_rows_judging.py  判「抓没抓到」的那几个入口与它们调到的函数：libtest 结果行的取法、本次未跑与设备丢失的认法、基线参数与单条基线结局、单条变异的结局分类、起 cargo 的命令、拷每片源码、判一行与判一批、覆盖复用判据。只 import base。驱动（这一份与别的份）的代码不进行键。
  crates_mutation_rows_support.py  自证的脚手架：样本仓与样本库的造法、格里读调用日志与改文件的小函数。假 cargo 与假 llvm 工具的正文（FAKE_CARGO、FAKE_LLVM_TOOL）在 base。
  crates_mutation_rows_static.py  变异表的静态核：拷贝范围的算法（path 依赖、--manifest-path、include 宏）、锚点失效、替换文里没绑定的标识符、转义字面量、release 补行与验证包补行，连同这几样在样本仓上的自证格。
  crates_mutation_rows_units.py  几个子进程按内存配速同时跑的个数、判定库与运行记录目录、行键（KeyContext）、复用判据（baseline_reuse_problem 在 judging），连同这几样的自证格。
  crates_mutation_rows_baseline.py  基线的切法：按批与按块切行、长测试单列、派变异的次序、行的分组，连同这几样与判结局那几样（在 judging）的自证格。
  crates_mutation_rows_targets.py  工作进程的停法、片号与编译目录（共用目录、片号锁、占盘、磁盘余量、上限清理），连同这几样的自证格。
  crates_mutation_rows_memory.py  内存：任务内存上限、进程树用量、每个工作进程的峰值（量、记、读、除）、内存准入、工作进程数与跑着加开，连同这几样的自证格。
  crates_mutation_rows_runs.py  起 cargo 与收输出：边跑边收、边跑边报、卡池目标、临时 prune-server、记覆盖、写用时，连同这几样与起 cargo 那几样（在 judging）的自证格。
  crates_mutation_rows_pool.py  先编一份依赖（先编组、拷依赖产物）、工作进程的初始化、基线与变异与覆盖三种任务、进程池 run_pool，连同这几样的自证格。
  crates_mutation_rows_plan.py  派活之前：读选行、核表（prechecks）、分行（plan_rows）、开 GPU 时补 gpu 特性、点名测试够不着被改坏的函数（static-check 可达性）、判复用、牵连面，连同这几样的自证格。
  crates_mutation_rows_stage.py  一道的整个跑法：按项机读行、run_stage、复用哨兵、记覆盖的排程（CoveragePlanner）、llvm 工具、汇总报告，连同这几样的自证格。
  crates_mutation_rows_commands.py  命令行的各个子命令（static-check、library-directory、row-plan、plan、--list-items、--item、sample-tools、llvm-tools），连同直接调子命令的自证格。
  crates_mutation_rows_driver.py  自证的驱动：Selftest 类、格函数表、弄坏开关与格的对应表、挑格与并行、run_selftest 与自带的几格。
弄坏开关的代码在哪一份（开关表仍在下面各节，名字出现在哪一份的 break_is / with_break 里，那一条开关的代码就在那一份；自证核表时逐份读）：
  crates_mutation_rows_base.py：baseline-release-incremental-kept、build-environment-keyed、build-jobs-keyed、debuginfo-kept、emptyreplacementrefused、memory-max-not-hard-cap、profile-keyed、release-incremental-unset、toolchain-version-keyed
  crates_mutation_rows_judging.py：anchor-mid-run-asserts、baseline-batch-composition、baseline-by-tree-snapshot、baseline-red-keeps-caught、cargoconfignotcopied、device-lost-ignored、given-states-count-ignored、gpu-test-threads-unadded、gpu-unused-as-missed、link-failure-as-invalid、not-run-line-start-only、not-run-test-name-only、notrunasmissed、product-gpu-feature-unquiesced、red-not-recorded、split-result-line-unseen、test-runs-configuration-unpassed、uncounted-overwrites-counted
  crates_mutation_rows_static.py：forked-parts-by-pool、include-literals-from-crate-root、manifest-path-ignored、no-default-features-unadded、path-dependencies-not-copied、release-unchecked、release-unforced、static-check-registry-from-script-repository、static-check-slices-misordered、undefinedidentifiersunchecked
  crates_mutation_rows_units.py：baseline-key-without-target、driver-and-library-keyed、key-context-kept-for-workers、key-parent-not-prewarmed、key-pool-not-frozen、key-token-values-unshared、library-outside-git-tolerated、mutated-test-service-code-keyed、mutated-unit-ignored、run-overrides-honored-outside-service、specification-lines-product-view、test-closure-ignored
  crates_mutation_rows_baseline.py：dispatch-by-crate-only、heavy-batch-row-dropped、heavy-batch-unsplit、long-baseline-not-isolated、timingsignored、unknown-timing-merged-as-short
  crates_mutation_rows_targets.py：depsnotpruned、diskbudgetbycap、disknotchecked、extratargetskept、inusetargetremoved、local-shard-numbering、overcapdispatched、overlapunchecked、runtargetkept、runtargetopenfilesunchecked、service-build-directory-ignored、target-measured-every-cell、targetcapignored、targetdefaulttmp
  crates_mutation_rows_memory.py：jobs-feedback-ignored、memory-cap-ignored、new-package-run-single-worker、sample-root-peaks-recorded、task-memory-constants-skewed、unmeasured-package-not-probed-alone、unmeasured-target-borrows-package-peak、unmeasured-worker-peak-guessed、unreadable-worker-peaks-guessed、worker-peaks-unkeyed-by-jobs、worker-peaks-unkeyed-by-package、worker-peaks-unkeyed-by-target、worker-peaks-window-unbounded、worker-tree-self-only、workers-beyond-rows、workers-by-processor-half
  crates_mutation_rows_runs.py：baseline-on-temporary-service、buildprofilesunset、computing-cell-baseline-unrecognized、computing-cell-coverage-unrecognized、computing-cell-prefix-renamed、computing-line-unwritten、computing-line-without-output、coveragenotrecorded、exactduplicated、gpu-test-threads-all-one、heartbeat-prefix-dropped、keepmutationtmp、lifecycle-filter-any-stream、lifecycle-lines-kept、lifecycle-prefix-loose、lifecycle-prefix-renamed、lifecycle-tail-dropped-on-stop、mutation-run-on-real-service、progress-line-only-on-output、progress-line-unwritten
  crates_mutation_rows_pool.py：baseline-one-at-a-time、baseline-timing-unwritten、broken-pool-looked-up-late、cargoconfignotcopied、compile-per-worker、dead-worker-cause-unprinted、gpu-cells-dispatched-in-parallel、heavy-batch-unsplit、lifecycle-lines-kept、long-baseline-not-isolated、narrowed-verdicts-recorded、plain-cells-cards-visible、records-written-at-end、seed-copy-serial、seed-copy-threads-unregistered、seedfreshacrossgroups、seedfreshunchecked、seedmtimekept、shardrootkeptonsignal、shardrootnamedlate、unknown-timing-merged-as-short、worker-failure-unconverted、worker-peaks-unrecorded、workspaceunitsshared
  crates_mutation_rows_plan.py：anchor-stops-table、gpu-route-feature-unadded、impact-row-dropped、key-context-kept-for-workers、nobaseline、reachability-closure-unchecked、reachability-cpu-configuration-ignored、reachability-cpu-configuration-test-code-only、reachability-edges-as-units、reachability-fallback-roots-ignored、reachability-refused-unjudged、reachability-rows-unsliced、reachability-source-text-ignored、reachability-stops-ignored、reachability-stops-unverified、row-key-collision-unrefused、rust-file-replaced-on-entry-module、workspace-files-unsliced
  crates_mutation_rows_stage.py：baseline-by-tree-snapshot、item-done-reused-silent、item-done-seconds-whole-run、item-plan-reusable-zero、link-failure-not-red、llvmversionunchecked、memory-cap-exits-before-fewer-workers、memory-cap-short-self-exit、narrowed-sentinel-drawn、narrowed-sharding-allowed、narrowed-verdicts-recorded、nobaseline、reuse-lookup-skipped、rocksdb-entry-unchecked、term-sent-unwaited、test-runs-configuration-unpassed、uncounted-forced-observed、unfiltered-rows-unchecked、unsharded-rows-judged、worker-jobs-held-after-memory-cut、workers-not-grown-after-measuring
  crates_mutation_rows_commands.py：anchor-rows-planned、item-plan-selection-ignored、listitemspipeunhandled、narrowed-sharding-allowed、rowplanselectionignored、staticusagebare、tablemissingok
  crates_mutation_rows_driver.py：cell-child-inherits-parent-process-state、cell-heaviest-first-ignored、cell-jobs-by-task-cap、cell-jobs-by-threads-only、cell-leftover-processes-kept、cell-limit-ignored、cell-outcome-dropped、pool-worker-caller-configuration-dropped、selftest-break-cells-ignored、selftest-break-table-entry-dropped

子命令（<根> 是被判仓的根）：
  static-check <根> [--table <变异表>]
                                        crates-mutation-replay 派活之前判的每一样（row_problems，crates-mutation-replay 的 prechecks 调的是同一个函数）连同拷贝范围 ②，
                                        每处一行（制表符分隔），退 0 一处都没有、退 1 有、退 2 读不了登记表或变异表：
                                          MALFORMED <行号> <变异名> <为什么>             这一行不成形（见上）；另有一样只 static-check 判（unfiltered_rows，crates-mutation-replay 派活之前不判）、也打这一类：
                                                                                         第五段 `--` 之后没有测试名筛选词（基线与每一次变异都把点名的测试目标整个跑一遍）。判的是本模块所在的仓、--staged 的临时树，
                                                                                         样本根里放了 .mutation-filter-words-judged 的；别的样本根不判。弄坏开关 unfiltered-rows-unchecked：一律不判
                                          ANCHOR   <行号> <变异名> <为什么>              原文在「文件」那一段指的源码里不是恰好命中一次，或文件不在、读不了
                                          NAME     <行号> <变异名> <测试名>              必须红的测试名不是一个用例名（带了正则符号或空白）
                                          ROW      <行号> <变异名> <文件>                ① 变异表这一行改的文件不在 crates-mutation-replay 每片拷的范围里
                                          STRAY    <行号> <变异名> <段名> <转义>         原文或替换文里有 \\n 以外的反斜杠转义
                                          LITERAL  <文件> <行号> <字面量> <解析到的路径> ② crates/ 下 .rs 里以 "../ 起头、解析之后逃出 crates/、又不在拷贝范围里的字符串字面量
                                                                                         （include_str!(…)、include_bytes!(…) 的实参照源文件所在目录解析，别的照所在 crate 的根解析；
                                                                                         只 code-source-discipline 判，crates-mutation-replay 靠基线）
                                          UNDEFINED <行号> <变异名> <文件> <标识符，逗号分隔>  替换文施加之后新出现、被改文件与它所在的包的 .rs 里都一处没有的标识符
                                                                                         （replacement_undefined_identifiers；替换文自己绑定的、方法与路径里的段、宏、属性里的、_ 起头的、
                                                                                         标准库预导入与基本类型不算；只 static-check 判，crates-mutation-replay 派活之前不判、套用后编不过记无效）
                                          IDENTIFIERS_CHECKED <判了标识符的行数> <没判的行数（文件不是 .rs 或锚点失效）>
                                          RELEASE  <行号> <变异名> <依据，；分隔>                命中重测试目标而第五段没写 release（见「重测试目标只许 release」；--profile release 也算写了）：
                                                                                         有这一行退 1，这几行之后一行「     → 怎么办：…」写出路（第五段补上 --release）；
                                                                                         判的是表里每一行（行数在 CHECKED 那一行），只 static-check 判，crates-mutation-replay 派活之前不判、照补 --release 起；
                                                                                         登记表读不了打 ERROR 行与「→ 怎么办」、退 2
                                        另判「点名测试够不着被改坏的函数」（判法见文件头那一节，一律判），打这几种行：
                                          UNREACHABLE <行号> <变异名> <closure 或 gpu-route> <被改坏的单元，、分隔> <点名测试> <门禁给这一行用的 cargo 参数>
                                                                                         点名测试在门禁实际的特性与多机配置下调不到被改坏的函数；有这一行退 1，
                                                                                         这几行之后一行「     → 怎么办：…」写出路
                                          REACHABILITY_EXEMPT <行号> <变异名> <cpu-configuration 或 gpu-tables-refused> <凭据单元>  照豁免认了、不判的行
                                          REACHABILITY_UNJUDGED <行号> <变异名> <为什么>   判不了的行（锚点失效、被改坏的单元或点名测试拒算、闭包算不成、闭包拒算而已收部分里没有被改坏的单元）
                                          REACHABILITY_CHECKED <判了的行数> <没判的行数> <豁免的行数> <截过闭包的行数> <多机配置> <ENABLE_GPU=1 或 GPU 没开>
                                          REACHABILITY_NOT_JUDGED <为什么>               选路表、截断表、豁免表点名的函数在 <根> 的源码里一个都没有（门禁样本那种小根）：这一趟没判可达性，
                                                                                         照实报，替换上面 REACHABILITY_CHECKED 那一行（门禁成功句写「可达性本次未判」，不算判过）
                                        选路表、截断表、豁免表里有的函数在 <根> 的源码里找不到（不是一个都没有）、多机配置读不了，打 ERROR 行与「→ 怎么办」、退 2
                                          COPIED   <拷贝范围，空格分隔>
                                          CHECKED  <成形的行数> <.rs 份数> <逃出 crates/ 的字面量处数（在范围里的与不在的合计）>
                                        门禁 code-source-discipline 调它；--table 不给就读 <根>/crates/mutations.tsv（不在当空表）；
                                        拷贝范围（COPIED、ROW）与重测试目标（RELEASE）读 <根>/.claude/gate.d/stage-inputs.tsv，不读本模块所在那一份仓的：<根> 里没有那张表、
                                        或表里没有 crates-mutation-replay 那一行，都打 ERROR 与「→ 怎么办」、退 2；套补丁的临时树要把这张表一起放进去；
                                        参数不对（缺 <根>、--table 缺值、<根> 不是目录）与 --table 指的文件不在都打 ERROR 行（原因）、
                                        下一行「→ 怎么办」，退 2；变异行的 --manifest-path 写坏了（见「--manifest-path」）同样打 ERROR、退 2
                                        并行（只改快慢与内存，判据与输出的每一行同逐行逐份算的一样）：被改文件每份只读一遍（anchor_problems_by_line），
                                        锚点与标识符、可达性两段共用；逐份、逐行互不依赖的几段切片并行（static_check_slices：逃出 crates/ 的字面量按 .rs 份，
                                        UNDEFINED 先按被改文件与包各切一次词、再按行细判），路数取 key_workers（GATE_MUTATION_KEY_WORKERS 或每台默认重活线程数，
                                        取不到打 ERROR 退 2），不到 PARALLEL_MINIMUM 份就地算，并回的次序同逐份的次序；
                                        可达性那一段逐行的被改坏的单元与点名测试按行切片并行（reachability_resolved_rows：要用的工作区 fork 之前在本进程里切好、冻进永久代，
                                        子进程只交串与 UnitReference，本进程补上形式化视图的记号、照（目标, 全名）取回本进程的点名测试单元）；被改坏的文件 row_view 与
                                        mutated_units 共用单元键记着的那一份（lib_mutation_units.py 的 UnitKeys.source_bytes，每份只读一遍）；
                                        可达性那一段点名测试的闭包照 KeyContext 那一份 precompute_closures_from_shared_parent 算（fork 之前共用词法单元、冻进永久代），
                                        截过的闭包 fork 之前同样冻，几行共用的闭包成员集合一组判完就放（reachability_plans_by_closure）。
                                        建工作区（lib_function_units.py 的 RustWorkspace）的每份 .rs 按文件切片并行切（prebuilt_workspace：子进程交切好的 RustFile，
                                        本进程照原样建工作区、文本的 sha256 与包目录对得上才取，剥法、限定名与索引照旧在本进程里做）；拷贝范围的闭包（lib_repository_copy.py 的 copy_closure）串行。
                                        标准错误末尾打一行「static-check 分段用时」（各段与可达性里各段的挂钟秒），标准输出不变。
                                        弄坏开关 static-check-slices-misordered：切片并回时片的次序倒过来；reachability-rows-unsliced：可达性逐行那一段不切片；
                                        workspace-files-unsliced：建工作区不按文件切片；rust-file-replaced-on-entry-module：切好的 RustFile 的替身换在入口模块上
  library-directory <根>                这棵树此刻那一份判定库的库目录名（prune-server 里那份库的名字与命名空间，本地没有文件）；算不出退 2
  row-plan <根> [--rows <选行文件>] [--progress]
                                        双机分片的驱动（scripts/mutation/mutation-shard-run.sh）分行用：每一条成形的变异打一行
                                          <行号> <行键，拒算写 -> <这一趟会不会复用，1 或 0> <上一次现跑的秒数，没记过写 -> <anchor-broken 或 ->
                                        （制表符分隔；「会不会复用」与 run_stage 同一份判法 row_decision；
                                        锚点失效的行（见「锚点失效」）不算键，打「<行号> - 0 - anchor-broken」）；
                                        带 --rows 时只对选行文件（一行一个行号，写法同 GATE_MUTATION_ROW_SELECTION）点名的行算键、判复用（与 run_stage 的选行模式同一份 plan_rows、decide_rows），
                                        别的行只给用时（键写 -、复用写 0）：点名一行不付整张表算键的代价，驱动分行取没记过用时的中位数仍取全表；
                                        stderr 打一行「  · row-plan：算了 <N> 行的键（…）」，N 是这一趟真算键的行数（点名两行就是 2）；
                                        末尾带 --progress（合并那一步用，写法见下面「合并那一步的分步进度」）就把核锚点、现算行键、读库、读用时表四步分步打进度；
                                        退 0；读不了表、上限写错、选行文件不对、算不出行键退 2；
                                        环境里设了缩域变量（文件头「缩过域的一趟」）打「  ✗ 缩过域（…）的变异不分片…」与出路、一行都不列、退 2
  plan <根>                             只读：不编、不跑、不写，逐行打「<行号>\\t<变异名>\\t复用|现判|拒算|锚点失效\\t<栏>\\t<细节>」（栏见「复用判据」，细节里列变了哪几个单元），
                                        末行「合计 <N> 行：复用 <M> 行、现判 <K> 行（<各栏的数>）、拒算 <R> 行、锚点失效 <A> 行」。退 0；读不了表、上限写错、算不出行键退 2
  --list-items <根>                     逐行打可点名的变异：「<变异名>\\t第 <行号> 行，改 <文件>」，锚点失效的那一行末尾接「；锚点失效：<为什么>」，不编、不跑（与门禁 --list-items 同一份）；
                                        末行合计这一趟键对不上（要现判）、对得上（复用）、拒算、锚点失效的各几条，锚点失效的逐行点名（行号与变异名）；
                                        再一行报牵连面（见「牵连面」）
  --list-items --with-keys [--rows <选行文件>] <根>
                                        另在逐行之后、末行合计之前，每项打一行 ITEM_PLAN（见「按项机读行」）；键没算出来退 1、不退回只列项名；
                                        --rows（门禁把 --item 点名的几行换成选行文件交来）只列、只算点名那几行的键与闭包，成本随点名的行数走，不随整表走
  --item <行号或变异名> [--item …] <根>  只判点名的几行：交给门禁 crates-mutation-replay.sh --item（同一份选行模式，点名单跑与整张跑时那一行判定相同，
                                        判定库照写、照复用）；点名的不在表里退 2。起 cargo、写判定库，经统一任务服务交
  sample-tools <目录>                   把自证用的假 cargo 与假 llvm 工具（FAKE_CARGO、FAKE_LLVM_TOOL）写成可执行文件放进 <目录>：cargo、llvm-profdata、llvm-cov。
                                        门禁 crates-mutation-replay 的样本档（样本根有 .crates-mutation-replay-sample-tools/ 时）把它放到 PATH 最前面、
                                        GATE_MUTATION_LLVM_TOOLS 指到它，样本 crate 里 `// TEST …` 与 `// COMPILE-ERROR-IF …` 几行就是假 cargo 判红绿的依据（写法见 FAKE_CARGO 头上的说明）；
                                        不起 cargo、不编译。退 0；<目录> 写不了退 1，用法不对退 2
  llvm-tools <根>                       打找到的 llvm-profdata、llvm-cov 与它们的 LLVM 版本；找不到或与 rustc 的 LLVM 版本对不上退 1、给出路
  --selftest [--cells <格函数名>[,…]] [--jobs <N>]
                                        自证（假 cargo、假 llvm 工具，在临时仓里跑 run_stage；格数由成功行现算）；
                                        有格的前提没造出来（并发领片号那一格两次调用没重叠）记本次未跑、逐格列出，别的格都对时退 77，不判绿；
                                        挑格与并行见「--selftest 的挑格与并行」

拷贝范围 = 被判那棵树的共用拷贝范围（copied_with_path_dependencies：scripts/hosts/lib_repository_copy.py 的 copy_closure，带测试代码扫——点名的测试在副本里跑：
  共用清单加 path 依赖指到的目录（例：验证包常在依赖的仓根 prune-server/）、编译期与运行期读的仓根文件连同它们 source、起的；那份文件头「闭包」），
  再加 .claude/gate.d/stage-inputs.tsv 里 crates-mutation-replay 那一行登记的路径（static-check 读被判那棵树 <根> 里的那张表，<根> 里没有它打 ERROR 退 2、不回落到本模块所在那一份仓的
  （judged_registry_root）；run_stage、plan、row-plan、--list-items、--item 读本模块所在那一份仓的：被判的就是这一份仓或带着自己脚本的副本，判别力样本的目录里没有登记表；
  经 declared 声明给 copy_closure，为什么是 REGISTERED_COPY_RANGE_REASON）；闭包算不成（path 依赖指到树之外、编译期读的文件认不出或不在）拒绝，
  run_stage 一条都不跑、static-check 打 ERROR 退 2，出路另写 <根> 是仓副本、缺的是闭包里的仓根文件那一种（照 minimal-copy.py 另建副本，COPY_REBUILD_HOWTO）。
  多机配置怎么交给副本里真跑的测试见「真跑测试的格把多机配置交给测试」一段，只编不跑的不带，
  再加变异行 --manifest-path 指的包目录里还不在范围里的（copied_with_manifest_directories，见「--manifest-path」；连同它的 path 依赖），
  底下每一段都不叫 target（各 crate 自己的编译目录）。crates-mutation-replay 照它拷每片的源码副本，code-source-discipline 照它静态判；判法只有 is_copied_into_each_shard 一处。
  ② 怎么解析：include_str!(…)、include_bytes!(…) 紧跟着的字面量照源文件所在目录解析（rustc 照这个目录找，INCLUDE_MACRO_BEFORE_LITERAL 认宏名），
  别的字面量照所在 crate 的根解析（cargo test 在包目录里起测试，运行期读的相对路径落在包目录底下）；解析到 crates/ 之内的不报。
  ② 管不到的：路径拼在变量里、环境变量给的、format! 拼出来的、不以 "../ 起头而运行期再 join("..") 的；#[path = …] 不算（编译期带进来的源文件，
  副本里编得过就在）；指到仓根本身（"../../" 这一类）按不在范围里算（之后拼上的是哪一份判不出）。
  弄坏开关 include-literals-from-crate-root：include 宏的实参也照所在 crate 的根解析。

锚点失效（anchor_problem，row_problems 的 ANCHOR 与 run_stage、plan、row-plan、--list-items 同一份判法）：原文在被判仓里「文件」那一段指的源码里不是恰好命中一次，
  或文件不在、读不了。按行判，不让整张表退出：那一行记它自己的判定「锚点失效」（不起 cargo、不算键、不写判定库、不复用，整道判红），别的行照判、照入库；
  「计数：」那一行有「锚点失效 N 行」一栏，判红的那一段逐行点名（行号、变异名、为什么）；选行（--item、双机分片的驱动）同样按行。
  派活之前另几样（不成形、测试名、拷贝范围、转义）照旧有一行就整道判红、一个工作进程都不起，那时锚点失效的行连同它们一起列出。
  row-plan 给锚点失效的行写「-」键与 anchor-broken，双机分片的驱动不把它们分给第二台。

--manifest-path（变异行点仓根工作区之外的独立工程，例仓根 Cargo.toml 的 exclude 列着的 prune-server/）：cargo 参数在 `--` 之前写
  --manifest-path <路径> 或 --manifest-path=<路径> 的行，包目录取那份 Cargo.toml 所在的目录（manifest_directory_of，相对 <根>；没写的是 <根> 那一份工作区）：
  · cargo 照原样带 --manifest-path 起（基线的批、变异、记覆盖都在副本根起，相对路径落在副本里那个目录）；基线按 `--` 之前整段分批，带不带、指哪一份各成一批；
  · 拷贝范围：那个目录的 Cargo.toml 已在范围里（仓根 prune-server/ 经 验证包的 path 依赖已拷）不动，不在就把那个目录补进来，连同它的 path 依赖（copied_with_manifest_directories）；
  · 单元键照那个包的源码算（unit_keys_under_manifest）：lib_mutation_units.py 的 UnitKeys 以包目录为根，被改坏的单元、点名的测试（在那个包的编译目标里找）、
    闭包都在它上面算，单元的文件照仓根起的路径写；被改的文件不在那个包底下的这一行拒算；
    记覆盖的工作进程与覆盖复用判据照同一份单元键核；
  · 写坏了拒：写了不止一次、后面缺路径、绝对路径、文件名不是 Cargo.toml、落在 <根> 之外、文件不在——run_stage 一条都不跑（prechecks）、
    static-check 打 ERROR 退 2，plan、row-plan、--list-items 把那一行记拒算。
  弄坏开关 manifest-path-ignored：不认 --manifest-path。

牵连面（impact_summary，只报数、不判红）：每一行的键依赖的单元 = 点名测试的单元闭包（KeyContext 为这一行算好的那一份，不另算）连同被改坏的单元；
  一个单元进了几行的这一份，改它一次就作废几行的判定。成功句之后与 --list-items 末了各打一行
  「牵连最大的函数单元：<文件>::<限定名> 进了 N 行的闭包（前 5：<单元> N、…）；一次改动最坏牵连 N 行」，单元按（种类、文件、限定名）认，
  N 相同按单元名排；拒算与锚点失效的行没有闭包，不算进，那一行末尾写明算了几行、没算几行。


复用：每行一条判定，行键只有两样：被改坏的那一处（函数单元指纹、原文、替换文）与点名测试（单元闭包键、选中它的开关与特性），
  不带变异名、路径、测试名；没有管整库的版本号、没有版本目录；一份库，两台判出的都经 prune-server 写进同一份（分片驱动 scripts/mutation/mutation-shard-run.sh
  合并时按此刻的行键核两片判的行在服务里读得到，它文件头「合并」）。
  没有整道全绿标记，不问 stage-must-run.sh。单元的切法、指纹与闭包在 scripts/gate/lib_function_units.py，这里经 scripts/mutation/lib_mutation_units.py 调
  （mutate.sh 的按条库也用那一份，文件头写了单元、工具链行与库的存放；工具链行、编译器版本行与依赖版本行哪张变异表的键平常都不收）。

不进键的：驱动（scripts/mutation/ 里 crates-mutation-rows.py 与 crates_mutation_rows_*.py 的代码：判结局、起 cargo、拷每片源码、派活的函数）、算键库
  （scripts/mutation/lib_mutation_units.py 与 scripts/gate/lib_function_units*.py 的代码：切单元、收闭包、算指纹的函数）、rustc 与 cargo 的版本、
  Cargo.lock 里的依赖版本、编译档（各档的 debug、incremental、opt-level、codegen-units 这类设置）、线程数（jobs、CARGO_BUILD_JOBS 与各个线程数变量）都不进行键、基线键、覆盖号：
  改它们不作废已判的行，只被改坏的那一处与点名测试的闭包变才作废。
  构建环境（编译器与链接器相关的环境变量 RUSTFLAGS 一类、cargo 配置的 build / target / env、根清单的 lints）一样都不进：键里留下的只有被改坏的那一处（单元指纹、原文、替换文）与点名测试
  （单元闭包键，连同选中它、定 cfg 的开关与特性：--release、--target、过滤词、libtest 参数、特性，闭包照它们收）。
  测试服务代码（scripts/gate/test-service-code-paths.tsv 登的：任务服务、判定库服务、GPU 批处理验证服务、算键的函数单元图、GPU 判器与它的 WGSL 内核这些；D13（验证路线） 已定项 17「三类代码」
  表测试服务代码那一行：不进任何键，改它不让它自己的测试与变异行重判）也不进：点名测试的闭包经 lib_function_units.py 的 closure_key，闭包里落在测试服务代码里的单元不出 unit 行与
  data 行（closure_key 排；点名的测试自己落在测试服务代码里的，入口单元的指纹照进键）；被改坏的那一处本身落在测试服务代码里的，行键里不放那个单元的指纹（上面「行键」的被改坏的单元）。
  撞键拒算的口径照旧：落在测试服务代码里、原文与替换文相同、点名同一条测试的两行，行键相同、拒算。
  自证 selftest_test_service_code_key_cell 核改被改坏的测试服务代码函数、改点名测试调到的测试服务代码函数，每一行的行键、基线键、覆盖号都不变，改点名测试自己只有点名它的那一行变；
  selftest_mutate_plan_test_service_code_cell 核 mutate.sh 的键同一件事（弄坏开关 mutated-test-service-code-keyed：被改坏的单元落在测试服务代码里也把指纹放进键）；
  闭包里测试服务代码的单元不进键那一格另借 lib_function_units.py 的弄坏开关（环境里设 FUNCTION_UNITS_BREAK=test-service-code-keyed）证红，本模块没有自己的开关。
  自证 selftest_driver_and_library_not_in_key_cell 核改驱动与算键库的函数体行键不变、改被改坏的函数行键变（弄坏开关 driver-and-library-keyed：仓根下驱动与算键库那几份文件的内容并进三把键）；
  selftest_debuginfo_cells 核改编译档与线程数行键不变（弄坏开关 profile-keyed、build-jobs-keyed：KeyContext 另收 lib_mutation_units.py 的 toolchain_lines 交的行，再加上编译档或线程数）；
  selftest_build_environment_and_version_cells 核假 cargo 的 -V 换一行、改 RUSTFLAGS 与几个编译器环境变量、cargo 配置的 [build] / [target] / [env]、根清单的 lints、
  Cargo.lock 里点名外部包的版本，行键、基线键、覆盖号都不变，没有 rustc 与 cargo（PATH 里没有）也算得出行键，改点名测试闭包里的函数行键变
  （弄坏开关 build-environment-keyed、toolchain-version-keyed、dependency-versions-keyed：前两个让 KeyContext 另收 toolchain_lines 交的行，后一个在 lib_mutation_units.py 里）；
  selftest_unit_key_cells 里「改 rustc、cargo 版本之后行键不变」「改编译器的环境变量 RUSTFLAGS 之后行键不变」两格核同一件事。
  覆盖落单元的判法（lib_mutation_units.py 的 UnitKeys.executed_units）与覆盖复用判据（baseline_reuse_problem）照旧：覆盖记录里每个执行过的单元的指纹逐个对此刻的源码。

行键（KeyContext.components，文件头「复用」那一句的逐样）：
  被改坏的单元   原文所在那份 .rs 里与原文重叠的最里层单元（fn 项；函数外的项按项单元：const、static、struct、impl 头、宏……）的（种类，指纹）：
                 所在文件与限定名不进键（D13（验证路线） 已定项 17：路径、名字、行号不进键）；
                 原文不落在任何单元上拒算；.wgsl 按 fn 切（原文所在的 wgsl fn，不在 fn 里的取它自己的常量、struct、绑定声明单元，取不到拒算，
                 不退回整文件）；别的不是 .rs 的（litmus/ 这类测试读的数据文件）取整份字节的 sha256；
                 落在测试服务代码里的单元（lib_function_units.py 的 is_test_service_code，登记表 scripts/gate/test-service-code-paths.tsv；文件头「测试服务代码」）连指纹也不带：
                 这一组里不放它（keyed_mutated_references），改那份测试服务代码不让这一行重判；这一行的行键只由原文、替换文、点名测试的闭包、开关与特性定
  原文与替换文   原样（\\n \\t \\\\ 转义）
  点名测试       cargo 参数选中的目标里那条 #[test] fn 单元的闭包键（入口 = 那条测试函数，lib_function_units.py 的 rust_closure_of，
                 带上 cargo 参数里的特性）；认不出是哪一个单元、或闭包拒算，这一行拒算
  开关与特性     cargo 参数里不是名字的开关（--release、--target、--include-ignored、过滤词与 libtest 参数这一类）与特性：它们选哪条测试跑、点名测试的闭包按哪一组 cfg 收
                 （--release 不带 debug_assertions、特性开关 cfg(feature)、--target 定 cfg(target_*)），属于点名测试的闭包那一样，不是构建环境；
                 -j、--jobs、--offline、--quiet、--color、--message-format 不进（parse_cargo_arguments 的 SWITCHES_NOT_AFFECTING_OUTCOME）
  不进的         环境变量与配置文件里的构建环境（编译器与链接器相关的环境变量、cargo 配置的 build / target / env、根清单的 lints）、rustc、cargo 的版本、
                 Cargo.lock 里的依赖版本、编译档（各档设置）与线程数（jobs、CARGO_BUILD_JOBS）
  拒算的行每趟照跑、不复用、不写记录，成功句另报「拒算 R 条」，stderr 逐行写为什么。
  撞键就拒算（refuse_colliding_row_keys）：表里两行以上算出同一把行键（两份内容相同的函数各改一处、点名同一条测试），
    这几行拒算：照跑、不复用、不写记录；stderr 逐行写「拒算（行键撞键：与第 N 行同键）」，成功句末尾另接「；其中撞键拒算 K 条：第 <行> 行拒算（行键撞键：与第 N 行同键）、…」。
    基线键与覆盖号不带被改坏的单元，不跟着拒算（两行各自的基线照常复用）。选行（--item、双机分片的驱动、row-plan --rows）只对选中的行算键时，
    整张表里原文与替换文都与选中的某一行相同的行（键撞得上的只有这几行）一起算键，撞键照整张表判。弄坏开关 row-key-collision-unrefused：不判撞键。

判定库（crates-mutation-replay 每条判完立刻写，下一趟按「复用判据」逐行判复用还是现判；存放在 lib_mutation_units.py 的 Library，读写都经 prune-server）：
  库的名字是库根的 singlefs-crates-mutation-library（只当 prune-server 里的名字与命名空间，本地不建、不读、不写；键前缀照
  scripts/prune-server/prune_server_client.py 文件头「键空间」：mutation-units[@命名空间]/singlefs-crates-mutation-library/）。库根是被判那棵树的 git common-dir
  （records_home）；取不到（不是 git 工作树）一条都不跑（run_stage 退 1、library-directory 退 2），不退回别处（弄坏开关 library-outside-git-tolerated 下拿被判的根当库根往下跑）。
  键里只用库名，哪棵仓不进键：主仓与仓副本（git init 过的）写进同一份；命名空间只在临时服务底下给（客户端 namespace_of，自证一趟一个库）。
  成功句末尾点名库在哪。服务连不上客户端退 75，这一道照 75 退出，不当「库里没有」现判：
    records/<行键>        一条变异的判定（判定、要打的那一行、判定日期、跑法、点名测试的闭包号、被改坏的单元号）
    baselines/<基线键>    一条点名测试在没改坏的副本上的基线（基线键 = 测试 fn 单元指纹、点名测试所在的包与测试目标、开关与特性）
    coverage/<覆盖号>     覆盖记录（覆盖号 = 测试 fn 单元指纹、点名测试所在的包与测试目标、开关与特性）：那条测试执行过的单元及各自的指纹，不带用例全名与快照号；
                          两个测试目标里字面相同的测试，基线与覆盖不互相顶替（弄坏开关 baseline-key-without-target：不带包与目标）；
    compositions/、units/  组成清单（每份点名测试的闭包）与单元表：作废时照它报变了哪几个单元
    index/<变异名与文件>  这一行上一次的键与几样组成的号：只用来说清现判的原因，不当复用的依据
  服务没有删的操作：库不按天数、个数、版本回收。记录里不写主机名。
  只给自证换的两个变量只在临时 prune-server 底下认（连哪一处与临时服务的标记都指同一份临时服务，判法同客户端的 selftest_service_problem；refuse_overrides_outside_service）：
    GATE_MUTATION_RUN_RECORDS_DIRECTORY=<目录>（运行记录的共享根取它）、GATE_MUTATION_MULTI_HOST_CONFIG=<文件>（真跑测试的格交给测试的多机配置）。
    不在临时服务底下设了任一个，点名设了的几个、一条都不跑（run_stage 退 1、library-directory 退 2）。
    弄坏开关 run-overrides-honored-outside-service 下这两个不在临时服务底下也照旧认。
  运行记录不进库（run_records_directory）：用时表 timings/<这一行六段原样的 sha256>（上一次现跑的用时，派活次序用；另一台的快慢不代表本机）
  与占盘记录 target-occupancy.tsv（文件头「磁盘余量」）放在共享根下的 crates-mutation-run-records<库的命名空间>/：共享根是多机配置
  （SINGLEFS_MULTI_HOST_CONFIG 指的那一份，没设取本模块所在仓的 multi-host.env，同 scripts/gate/admission.py 的 multi_host_configuration_file）里
  NFS_SHARED_ROOTS 点名的、不含本仓也不在本仓之内的几段里排在最前的一段（两台同一绝对路径，不经 ~ 取）；配置里没有这一项就整道退 2，不补缺省、不落到 git common-dir。
  GATE_MUTATION_RUN_RECORDS_DIRECTORY=<目录> 设了（只在临时服务底下认，见上），共享根取它。

复用判据（row_decision，按次序）：
  拒算            这一行算不出键，或与表里别的行撞键（见「行键」）：照跑，不复用
  复用            库里有这把行键的记录，判的是抓到
  上次没判成抓到  有这把键的记录，判的是别的档（没红、无效、基线不绿、本次未跑……只有抓到的复用）
  记红、撤绿：判出的每一档都照行键写 records/（judge_row），没红、无效这些不是抓到的盖掉同一把行键原有的抓到（弄坏开关 red-not-recorded：只记抓到）；
  不算数的结局（本次未跑、链接失败：VERDICTS_UNCOUNTED）不盖同一把行键原有的算数结论
  （抓到、没红、无效：VERDICTS_COUNTED），那一条不写、原有的留着；库里没有这把键的算数结论时照写（uncounted_verdict_keeps_counted_record；
  弄坏开关 uncounted-overwrites-counted：不算数的照写、盖掉原有的算数结论）。
  链接失败（verdict_of_run 的 linkfailed）：替换之后编过了、失败在链接——输出里有「error: linking with」与 rust-lld、没有 error[E<码>]，点名的测试一行都没出现。
  不记无效（无效说的是替换文编不过、叫人去改替换文），也不记没红；单列一档、这一趟判红，出路是删掉这一趟用的编译目录重跑这一行（多半是编译目录里别的命令编出的旧产物与这一趟对不上）。
  弄坏开关 link-failure-as-invalid：照无效记；link-failure-not-red：这一趟不因它判红。
  基线不绿的行另照它的行键记一条 baseline-not-ok、它的基线键记 not-ok（judge_batch；弄坏开关 baseline-red-keeps-caught：行键那一条不记），下一趟这几行现判、先跑基线。
  键对不上的那一行照索引里它上一次的那一份逐样比，栏报头一样变了的：被改坏的单元变了、点名测试的闭包变了、原文或替换文变了
  （索引里没有这一行的报「行新加」；三样都没变而键变了的报「键对不上」：cargo 参数里的开关或特性改了），细节里列变了的单元（改了 / 新进 / 出了 <文件>::<限定名>）。
  前提：用例对同一份源码执行的函数集合是确定的（随机源、线程、时序只改函数里走哪条分支）。
  基线 ok 那一条（要现判的行先看它的基线能不能复用，baseline_reuse_problem）：库里有这把基线键的 ok 记录，而且覆盖记录里那条测试执行过的单元
  （执行过的 fn 单元，连同它闭包里不是 fn 的单元：常量、类型、静态项、宏、数据文件）此刻逐个是那个指纹，或者点名测试的闭包键与记录里的相同；
  不按整树快照比（同一个文件里加一个没人调的函数、改别的测试，基线照样复用）。

分片（门禁 crates-mutation-replay 一律经服务分片判，它文件头「分片」）：键对不上的行只在分片的各片里现判（选行模式，GATE_MUTATION_ROW_SELECTION）；
  门禁不是一片的那一趟（合并那一步与 gate.sh 整轮）设 GATE_MUTATION_SHARDED_ONLY=1，run_stage 照「复用判据」逐行判之后，
  栏是行新加、被改坏的单元变了、点名测试的闭包变了、原文或替换文变了、键对不上的行有一行就整道退 1、一条都不跑，逐行点名（前 REASONS_SHOWN 行）；
  拒算的（没有键，分片判出来也导不出）、复用哨兵抽中的、上次没判成抓到的照判。弄坏开关 unsharded-rows-judged 下照现判。

缩过域的一趟（crash_domain_narrowing；.claude/rules/verification.md「缩域」那一条）：环境里设了崩溃域的缩域变量（名单与「设了就算、空值也算」同 scripts/gate/admission.py 的
  CRASH_CASE_NARROWING_VARIABLES：SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO、SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO）就是缩过域，run_stage、row-plan 与 scripts/mutation/prove-red.sh 同经这一个函数判：
  · 现判的不入库：工作进程那一份库一样都不写（unwritten_library：判定、基线不绿、基线 ok、覆盖），索引也不写，不记覆盖；组成清单照写（只是单元表，不是判定）。
    库里的判定都是全域下判的，缩域判出的「抓到」「没红」放进去，不缩域的下一趟就照它复用了。弄坏开关 narrowed-verdicts-recorded：照写。
  · 复用照库里全域下判的抓到（行键相同才复用，与不缩域同一份 row_decision）：同一把键全域判出抓到，提交时门禁认的就是它；缩域只少判几个崩溃状态，
    再判一遍只会得出更弱的结论（缩域下红了全域也红；缩域下没红说明不了全域）。基线 ok 同理：全域下绿，缩域下判的崩溃状态是它的一部分，照绿（推的：测试的断言只看判到的状态）。
  · 不抽复用哨兵（plan_reuse_sentinel 照 off 记一行原因）：强制现判出的是缩域下的结论，拿它比全域的抓到会把缩域不够当成复用失灵。弄坏开关 narrowed-sentinel-drawn：照抽。
  · 每一行现判出的结局（抓到、没红、无效、本次未跑、基线不绿）头一行末尾接「；缩过域（<变量=值>）、不算门禁通过、没入库」（narrowed_outcome_text），
    成功句与选行那一句写「缩过域（<变量=值>）、不算门禁通过、现判的没入库」，stderr 在判定库那一行之后另打一行；退出码不变。
  · 不分片：服务分片的计划与合并那一步调的 row-plan 退 2，分片的各片与合并那一步（TASK_SERVER_SHARD_PHASE 是 shard、merge）的 run_stage 退 2，红句与出路是 narrowed_sharding_refusal：
    各片判出的不入库，合并那一步按库核各片判的行就核不到。点名几行缩域证红走 prove-red.sh（不分片）；要门禁结论去掉缩域变量、范围声明写全域理由再交门禁。
    不经服务分片的核账（gate.sh 整轮、手跑，GATE_MUTATION_SHARDED_ONLY=1）照收：只复用、不现判键对不上的行。弄坏开关 narrowed-sharding-allowed：照收。
  记录里不写当时的环境（judge_row 写的字段见「判定库」）：库里一条判定是不是缩过域判的，从库里查不出。

覆盖记录（一条现判成抓到的行，它点名的测试此刻还没有一份执行过的单元都没变的覆盖记录时，在没改坏的副本上记一份，judge_coverage）：
    ① cargo test --no-run --message-format=json-render-diagnostics 带 -C instrument-coverage（环境里有 RUSTFLAGS / CARGO_ENCODED_RUSTFLAGS 就接在后面，
       没有就经 --config build.rustflags，与配置文件里的合并），编译目录是这个工作进程的编译目录加 -coverage；
       LLVM_PROFILE_FILE=<这一格的 TMPDIR>/build-profiles/build-%p-%m.profraw：cargo 没带 --target 时 RUSTFLAGS 也罩构建脚本与过程宏，它们在各自依赖包的源码目录
       （~/.cargo/registry/src/ 下）里起，不设这个变量就在那里写 default_*.profraw；
    ② cargo test -- <开关> --exact <用例全名>，LLVM_PROFILE_FILE=<这一格的 TMPDIR>/cov-%p-%m.profraw；点名用例要恰好一行 ok；
       行里 libtest 参数已带 --exact 的，开关里去掉它再补自己的，--exact 只出现一次（libtest 见两个报 Option 'exact' given more than once、退 101）；
    ③ llvm-profdata merge -sparse，llvm-cov export -format=text -skip-expansions，对象是 ① 编出来的本仓每个可执行文件（测试与 bin）；
    ④ 记：计数 > 0 的函数按它第一段代码区域的行落进的最里层单元（lib_mutation_units.py 的 executed_units），连同点名测试闭包里不是 fn 的单元，
       各自的（种类、文件、限定名、指纹）；有函数落不进任何单元的，这一份记成不全、不拿来复用。
    ①–③ 同变异一样直接起，不设上限与限时；有一步不成这一条测试不记（下一趟它的基线只按闭包复用），成功句另报「覆盖没记成 N 份」，不判红。
  llvm 工具：GATE_MUTATION_LLVM_TOOLS=<目录> 设了用它（双机分片的驱动设了就带到第二台），不设取 rustc --print sysroot 下 lib/rustlib/<host>/bin
    （rustup 组件 llvm-tools）；两样都要在、--version 的 LLVM 版本要以 rustc -vV 的「LLVM version」起头，不然这一道判红、一条都不跑，不退回不记覆盖。
  用时表：运行记录目录（文件头「判定库」末段，run_records_directory）里 timings/<这一行六段原样的 sha256>，记上一次现跑的用时；不进判定库。
  用时：从起 cargo 到它返回的秒数（seconds_running）。

上限、限时与卡住（records/2026-10-30-编译绕过内存总上限.md：工具里不限时，整条照旧交服务）：每条 cargo 直接起，不套内存包装、不设上限、不限时；
  一整条门禁交给统一任务服务（task-server/），内存上限由服务按记录给、撞了放大重排，死循环的变异由服务停整条任务（用时远超记录而日志又满 10 分钟不涨就停掉、打回）；
  不判「内存撞顶」「超时」「显存不够」：显存不够而红的照抓到记；GPU 设备丢失（点名的测试自己那一段输出里、或所有段之外有行首 GPU_DEVICE_LOST 的行）另算，记本次未跑（下面「本次未跑」）。

边跑边收、边跑边报（run_capturing_output；工作进程收 cargo 的输出，内存有界，跑得久的那一格任务日志照样涨）：
  收：两根管道各一条读线程按行读。stderr 里以 GPU_LIFECYCLE unix_microseconds= 起头的整行（crate singlefs-verification 的 gpu_work_trace 绕过 libtest 的捕获直接写 fd 2，
    task-server/cleanup-protocol.md 要它，一格能写出十几 GiB）不进交回的结果，只计条数与字节、内存里留最后 32 行；别的行照收，交回的 stdout、stderr 与
    capture_output=True, text=True, errors="replace" 收的逐字相同，滤掉过行的 stderr 末尾多一行汇总 `GPU_LIFECYCLE_DROPPED lines=<条数> bytes=<字节>`。
    判法不读这些行：结果行在 stdout（libtest），本次未跑记号（not_run_marks_in）、CRASH_POINT_REPLAY_GIVEN_STATES、error 行都不是这个前缀，
    output_tail 只进判定的说明文字。滤的只有 stderr、只认行首：stdout 里同样前缀的行与行中间夹着这几个字的行都照收（结果词被它隔开的 FAILED 照认）。
  被停：留下的行打进任务日志。这一进程接住了停的信号（父进程在 run_pool 里，先编一份依赖那几趟）时，对整组发 TERM、等它退之后打、再往外抛；
    信号是默认处理（工作进程，init_worker）时，打完照默认处理停，退出码照旧是被信号停。
  报：调用方的主线程照固定间隔 GATE_MUTATION_PROGRESS_SECONDS 秒（没设 60，output_progress_seconds；不是正整数 run_stage 拒）往 stderr 打一行，
    间隔点照起跑时刻往后数（第 k 行在起跑之后 k 个间隔，不随前一行打得早晚漂），子进程这一段有没有新输出都打（编译与跑测试可以十几分钟不出一个字）；
    读线程晚醒、跨过了几个间隔点的，补上跨过的那几行（每个间隔点一行，不丢）。每一行以 `TASK_SERVER_HEARTBEAT ` 起头（统一任务服务认的心跳行前缀，
    task-server/task_server_lib.py 文件头「命令自己打的两种登记行」）：服务判卡住看的「日志不涨」不把这些行算作在涨，一格编译或跑测试时一直只有它、CPU 也不涨，照样判卡住：
    `TASK_SERVER_HEARTBEAT MUTATION_WORKER_PROGRESS process=<进程号> elapsed_seconds=<这一格跑了多久> stdout_bytes=<字节> stderr_bytes=<字节> lifecycle_lines_dropped=<条数> lifecycle_bytes_dropped=<字节>
    anon_bytes=python:<字节>,test-binary:<字节>,rustc:<字节>,prune-server:<字节>,other:<字节> command=<命令的前 200 字>`。anon_bytes 是本进程整棵进程树按类别（process_kind）加总的 RssAnon。
    lifecycle_lines_dropped 数的是上面「收」那一段照规矩滤掉的 GPU 阶段现场行（不进交回的结果、最后 32 行留着），不是丢掉的进度行。
    心跳前缀是文本字面量，只写在 wait_for_output_readers 里（不做模块级常量）；服务那一头的前缀是 task_server_lib.py 的 HEARTBEAT_LOG_MARK，
    两头用的是同一个字串，自证「边跑边报…」两格照它的全文判。
  只认这一处的前缀：crates 那侧改了 gpu_lifecycle_line 的写法，自证「滤的前缀与 crate singlefs-verification…」那一格判错。
  在算：心跳行不算日志在涨，GPU 阶段现场行又被滤掉，一格跑几个小时任务日志就一行不涨、只剩 CPU 当活着的凭据；所以调用方的主线程另照门槛
    GATE_MUTATION_COMPUTING_LINE_SECONDS 秒（没设 300，computing_line_seconds；不是正整数 run_stage 拒）打在算行（output_computing_line），不带心跳前缀、服务算它日志在涨：
    `MUTATION_WORKER_COMPUTING cell=<这一格> targets=<测试目标，逗号分隔> tests=<过滤词，逗号分隔> elapsed_seconds=<这一格跑了多久> window_seconds=<离上一个门槛点多久>
    lifecycle_lines_dropped_in_window=<这段滤掉的条数> lifecycle_bytes_dropped_in_window=<字节> output_bytes_in_window=<这段子进程出的字节，滤掉的也算>`。
    这一格（computing_cell_label）照 TMPDIR 的名字认：mutation-row:<N>（变异表第 N 行）、baseline-batch:<N>（批首行是第 N 行的那一批基线）、coverage-group:<N>（记覆盖那一组），认不出写 -；
    建目录的前缀写在 judge_row（line-）、run_baseline_batch（baseline-）、judge_coverage（coverage-）各自的 tempfile.mkdtemp 里，前两个不能改成从认格名的一处取；
    只留认格名这一处前缀表，自证读这三个函数的源码，核它们建的目录名 computing_cell_label 都认得出。
    点名的测试（computing_tests_of）取命令里头一个 test 词往后的 --test <名>、--lib 与 `--` 之后的过滤词。
    门槛点照起跑时刻往后数，晚醒跨过几个点只打一行；从上一个门槛点到这一个子进程有新输出（滤掉的 GPU 阶段现场行也算，当作在算的读数）才打，
    一个字都没出的不打：真卡死的命令只剩心跳行、CPU 也不涨，服务照样判卡住。
  弄坏开关 lifecycle-lines-kept、lifecycle-filter-any-stream、lifecycle-prefix-loose、lifecycle-prefix-renamed、lifecycle-tail-dropped-on-stop、progress-line-only-on-output、progress-line-unwritten、heartbeat-prefix-dropped、
  computing-line-unwritten、computing-line-without-output、computing-cell-baseline-unrecognized、computing-cell-coverage-unrecognized、computing-cell-prefix-renamed（弄坏开关那一节末尾）。

合并那一步的分步进度（merge_step_open 起头的几个函数；scripts/mutation/mutation-shard-run.sh 文件头「三步」merge）：服务分片的合并那一步里，驱动先经 row-plan --progress 对各片判的行按此刻的源码现算行键、读判定库，
  之后门禁在同一步里调 run_stage 核账；这两处一步一对开始、结束行打 stderr（行首「  · <范围>·<步名>：」，范围在 row-plan 里是「合并」、在核账里是「合并·核账」，结束行带「完，用了 N 秒」与这一步的结果），
  一步里的一段算完（点名测试的闭包）另打一行带这一步已用几秒；逐行做的步（核锚点、现算行键、读库、读用时表，核账里另有预检、抽哨兵候选、核复用与基线、写组成清单）
  每过一个固定间隔（GATE_MUTATION_PROGRESS_SECONDS，没设 60 秒）打一行「在第 N 行、共 M 行」，行首是 `TASK_SERVER_HEARTBEAT `（同上面「边跑边报」，这种行不算日志在涨）；开始与结束行不带它。
  范围名放在环境变量 GATE_MUTATION_MERGE_PROGRESS_SCOPE（merge_progress_variable）：row-plan 带 --progress、run_stage 在 TASK_SERVER_SHARD_PHASE=merge 时自己设，不手设；没设什么都不打。
  GATE_MUTATION_PROGRESS_ROW_PAUSE_SECONDS 只给自证（驱动的自证「合并分步输出」两格）：逐行的步每过一行停这么多秒当假的慢读库，只在临时 prune-server 底下认。
  分步进度靠环境变量传范围、每个函数自己开账，不加参数。

基线（派变异之前，按批在没改坏的副本上跑一遍点名的测试）：
  批的键 = 这一行 cargo 参数里 `--` 之前那一段 + `--` 之后以 `--` 起头的开关；过滤词取这一批要跑基线的各行 `--` 之后非开关词的并集，
  有一行没有过滤词就整批不带过滤词。要单独带值的开关（`--skip 名字`、`--test-threads 4`）写成 `--test-threads=4`：分开写时那个值会被当成过滤词。
  与变异一样直接起 cargo，不设上限与限时。
  一行的基线结局不随这一趟拼进了哪几行而变（--item 只选几行、整张表一批几十行，同一行判同一个结论）：每行只看它自己的过滤词（连同 --exact、--skip=）选得中的结果行
  （row_baseline_output，选法照 libtest：带 --exact 名字逐字相同、不带是子串）。弄坏开关 baseline-batch-composition：照整批的输出判。
  每一行的点名测试在基线输出里要恰好对上一个用例、结果都是 ok，否则这一行记「基线不绿」：
  不算抓到、不算没红、它的变异不跑，整道判红。已有作数的抓到记录或基线 ok 记录的行不再跑基线。
  重基线批切块（baseline_chunks）：cargo 参数点名 GPU_TEST_THREADS_RELEASED_TARGETS 里的测试目标、或命中「重测试目标只许 release」（lib_release_only_tests.py）的批，
    批的键不动，同键的行按过滤词切块，每块至多 GATE_MUTATION_BASELINE_CHUNK_FILTERS 个过滤词（没设 16，baseline_chunk_filter_limit；不是正整数 run_stage 拒），派活时重的块排在别的基线批之后；
    同一个过滤词的行进同一块；有一行没有过滤词的批不切（它要跑全部用例，切了白跑几遍）。一块一个进程：块跑完进程退出，测试堆与输出都还回去；基线的块不起临时 prune-server、连调用方的那一份。
    判完就写判定库（judge_batch），被停只丢在跑的那一块，重跑同一条命令按基线 ok 记录只补没判的块。判定不变靠上面那一条：一行只看自己的过滤词选中的结果行。
    stderr 头一行的「现跑基线 N 批」数的是块，后面接「（其中长测试独占 K 批、没有用时独占 U 批）」；stdout 的「按 cargo 参数分 N 批」数的是批的键，不随切块变；基线记录的 batch_arguments 随块变，不进键。
    块之间共用调用方的那一份判定库服务：先跑的块判过的崩溃状态块写进去，同一批里后跑的用例照键读到。
    弄坏开关 heavy-batch-unsplit、heavy-batch-row-dropped；切与不切判定相同由自证对拍，baseline-batch-composition（照整批的输出判）下那一格判错。
  没有用时的与长的测试各占一块（baseline_chunks、split_rows_by_length，在重基线批切块之前先分）：一批里点名的测试按上一次用时分成三类：短的、没有用时的、长的；
    共用同一个过滤词的行是同一条测试（baseline_test_groups），没有用时的与长的每条测试单独一块，短的照上面的切块（重的批按个数切，轻的批整批一块）。块的次序是三段：
    短的块在前，没有用时的各占一块居中（照表序），长的各占一块在最后（上一次用时从短到长，先出结局的先出）。每块一个进程，后面的块起来之前前面块的结局不必等它。
    上一次用时是用时表（run_records_directory 底下 timings/<这一行六段原样的 sha256>，read_timing 读）里这一行的数：变异那一行判完写（write_row_timing），
    基线一块判完也写（write_baseline_timings，经 baseline_job）：这一块只是一条测试、它的行基线绿了、用时表里还没有那一行的，记这一块判完的秒数；
    多条测试合在一块的、整批没有过滤词的、基线不绿的、已有用时的不记。用时是从起 cargo 到它返回：含编译，是测试用时的上限估计，宁可多单列几块也不让短行陪长行等。
    一条测试里有一行不短于 baseline_long_test_seconds（函数体里写着门槛 900 秒；不做模块级常量）算长；不然有一行没有用时数就算没有用时
    （没判完过的行没有用时数；不知道长短的照长的办，不按登记表或目标名猜）；每一行都有用时、都短于门槛才算短。判完一次有了用时，短的下一趟回到按个数切的块里。
    没有过滤词的行所在的批不分（要跑全部用例）。GATE_MUTATION_BASELINE_LONG_TEST_SECONDS 只给自证：换门槛秒数，只在临时 prune-server 底下认。
    stderr：头一行与收尾一行的「现跑基线 N 批」后接「（其中长测试独占 K 批、没有用时独占 U 批）」；每块长测试起跑打一行「  … 基线长测试单列一块：<测试目标> 的 <过滤词>（依据；批首行是变异表第 N 行）」，
    每块没有用时的测试起跑打一行「  … 基线没有用时的测试单列一块：<测试目标> 的 <过滤词>（没有用时数据；批首行是变异表第 N 行）」。
    一行的基线键与行键不因切法变。
    弄坏开关 long-baseline-not-isolated：没有用时的与长的都不单列，照旧只按过滤词个数切；unknown-timing-merged-as-short：没有用时的照短的并进按个数切的块；
    baseline-timing-unwritten：基线一块判完不记用时。

本次未跑：点名的测试在本机没跑到被改坏的那一处时，打一行 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 后返回成功（GPU 对拍用例在没卡的机器上）。
  变异那一趟的 libtest 参数里补 --show-output（with_shown_output，插在第一个 `--` 之后）：成功用例的输出 libtest 默认不交出来；
  不用 --nocapture：单线程跑时它把用例的输出插进「test <名字> ... 」与结果之间（隔开的结果行由 joined_result_lines 接回，但整段输出会长得多）。基线那一批不补。
  点名的测试结果是 ok、输出里有这一族本次未跑行的，这一条记「本次未跑」（not_run_reason）：不算抓到、不算没红，逐条打出来；
  记号不要求在行首：用例自己没换行就打、接在别的输出后面、前面有颜色码、一行里有两处的同样认（not_run_marks_in：认法是 lib_libtest_result_lines.py 的 marker_pieces，
  整词、一行里每一处各是一截、颜色码先去掉；认的名字只有 NOT_RUN_NO_GPU_ADAPTER 一个，别的 NOT_RUN_<原因> 只用来划截；弄坏开关 not-run-line-start-only：只认行首的）；
  test= 对上点名的测试（只比 :: 分开的末一段）的那一处先取它的 reason=；都对不上的照样记本次未跑（崩溃枚举用例判段不跑时打的是
  test=crash_state_enumeration::<项名>，项名不是测试名），那一句写明 test= 是什么、对不上测试名：宁可记本次未跑，不记没红
  （弄坏开关 not-run-test-name-only：只认 test= 对得上的）；
  整道不因它判红，成功句写明抓到几条、本次未跑几条；一条抓到的都没有、只有本次未跑的退 77（选行模式照旧退 3 / 1）。
  GATE_MUTATION_NOT_RUN_RECORD=<文件> 设了，每条本次未跑往里追加一行（阶段把它转给 gate.sh 的 GATE_NOT_RUN_FILE）；写不进去整道判红。
  只有「抓到」复用，本次未跑的下一趟照跑。
  设备丢失（verdict_of_run 判，门禁与 scripts/mutation/prove-red.sh 同一处）：点名的测试 FAILED（退出码非 0），而下面两处之一有一行行首是 `GPU_DEVICE_LOST ` 的行
  （crate singlefs-verification 的 gpu_memory_ledger 用 println! 打，`GPU_DEVICE_LOST test=<当前线程名> card=<卡名> reason=<一句>`），记「本次未跑」：
  红的是设备丢失、不是断言，不算抓到、不算没红，走与 NOT_RUN_NO_GPU_ADAPTER 同一路（VERDICTS_UNCOUNTED：不盖行键原有的算数结论、逐条打出来、整道不因它判红、下一趟现判）。
  判次序（device_lost_line_of_test、device_lost_line_outside_sections）：
    点名的测试自己那一段（libtest 给它交出的「---- <全名> stdout ----」，named_test_stdout）有 ⇒ 本次未跑；
    所有段之外（不在任何一段里，归属不明）有 ⇒ 本次未跑；
    只在别的用例的那一段里有 ⇒ 抓到（点名的测试这一段没有它）；都没有（显存不够而红的没有）⇒ 抓到。
  不拆那一行的字段、不比 test=（线程名取不到时 test= 是 main 或 unnamed-thread，卡名里可以有空格）；点名的测试 ok 而带这一行（测试自己认下）照 ok 的判法走（本次未跑行、没红）。
  段与段外的切法（libtest_stdout_sections，named_test_stdout 同一份）：libtest 的 pretty 输出先是实时打印的部分（「running N tests」「test <全名> ... ok|FAILED」，
  没被捕获的输出夹在这里：不是点名用例这条线程派生的线程打的行、子进程的输出写到真标准输出，不在任何一段里），再是「successes:」「failures:」下各用例的段，最后「test result:」；
  一段从「---- <全名> stdout ----」那一行起，到下一个段头、「successes:」「failures:」或「test result:」为止，这几行本身与之后的行算段外；段里的行一行都不算进段外。
  输出里一段都没有（没带 --show-output、或带 --nocapture）时 named_test_stdout 交整份、段外也是整份，整份里有就算；变异那一趟固定带 --show-output、不带 --nocapture，走不到这一支。
  弄坏开关 device-lost-ignored：不认这一行。
  带 gpu 特性的格另有一种记本次未跑（给定状态全交 CPU 判），见下面「真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡」。

带 gpu 特性的格不领卡、不清场（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：
  卡与清场归 GPU 批处理验证服务，这一道不领卡、不挑卡、不清场：环境里有 GPU_CARDS_BY_PRIORITY 就原样交给起的 cargo，没有不设。
  这一道起的每一次 cargo 都直接起，不调清场包装；双机分片在第二台起的那一份同样。
  走崩溃判段的测试由验证包写待判行、等 GPU 服务判；测试自己要开卡的（GPU 对拍、卡池测试）开不了卡时打 NOT_RUN_NO_GPU_ADAPTER 或 GPU_DEVICE_LOST，
  照文件头「本次未跑」那一节（含设备丢失）记本次未跑，判法不另改。
上卡的格一格接一格（run_pool 的派活与工作进程里的 pool_cell_job；pool_cell_runs_on_gpu、card_hiding_environment）：
  进程池里起的 cargo test 是 gpu_test_run 认的那一种（开了上卡的特性、真跑测试）的格是上卡的格：基线的批照这一批的参数、变异照这一行的参数、记覆盖照这一组首行的参数判。
  上卡的格不进平常那一队、挪到最后：不上卡的都派出去、收回之后，一次只派一格（测试二进制之间一个接一个上卡，卡池的显存账按进程记：
  .claude/rules/verification-reference.md「测试线程」）；基线绿了才排进来的变异格、抓到之后才排进来的记覆盖同样进这一队。起的命令照原样，不另编、不改参数。
  弄坏开关 gpu-cells-dispatched-in-parallel：上卡的格与不上卡的同一队、照空位并行派。
  不上卡的格看不到卡（card_hiding_environment）：这些格起的每条 cargo 的环境里，VK_DRIVER_FILES 与 VK_ICD_FILENAMES 指到一份不存在的驱动清单（这一格自己 TMPDIR 底下的 no-vulkan-driver.json，
    不建它）、CUDA_VISIBLE_DEVICES=-1、去掉 VK_ADD_DRIVER_FILES；上卡的格的环境里这几个都不动。不藏 GPU_CARDS_BY_PRIORITY：
    它决定测试怎样读卡配置，藏了判定会跟着变（推的）。用这几个变量的依据：
    · 本仓建 wgpu 实例的两处都只开 Vulkan 后端、都用不读环境的 new_without_display_handle：crates/singlefs-background-gpu/src/background_gpu.rs:825 与 :836-839，
      crates/singlefs-verification/src/gpu_unit_checks.rs:630-633；wgpu 依赖只开 vulkan 特性（crates/singlefs-verification/Cargo.toml:24、crates/singlefs-background-gpu/Cargo.toml:14）。
      所以 wgpu 的 WGPU_BACKEND 一类变量对这两处不起作用（wgpu-types 的 new_without_display_handle_from_env 才读它们），wgpu 看得到哪些卡全由 Vulkan 加载器装的驱动清单定；本仓没有 CUDA 路径；
    · 加载器认 VK_DRIVER_FILES、VK_ICD_FILENAMES（换掉默认清单）与 VK_ADD_DRIVER_FILES（追加清单）：crates/singlefs-verification/src/gpu_unit_checks.rs:646-651 用户设了前两个之一就不再自己指 NVIDIA 的清单；
      crates/singlefs-background-gpu-tests/tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs:400-438 用同一套写法（指到不存在的清单、去掉 VK_ADD_DRIVER_FILES）造「没有独显」；
    · 量的：同样两个变量指到不存在的文件，vulkaninfo --summary 打 vkCreateInstance: Found no drivers!、vkCreateInstance failed with ERROR_INCOMPATIBLE_DRIVER，退 1；
    · CUDA_VISIBLE_DEVICES=-1 只为 CUDA 运行库进了子进程（构建脚本、被测程序起的别的工具）时也看不到卡（推的，本仓代码里没有读它的地方）。
  弄坏开关 gpu-cells-dispatched-in-parallel、plain-cells-cards-visible（弄坏开关那一节末尾），自证 selftest_gpu_cell_dispatch_cells。
  上卡的特性（GPU_TEST_FEATURES：验证包的 gpu，与产品 GPU 包 singlefs-background-gpu 真显卡对拍的 tests-on-a-real-gpu；
  --features / -F 里写这些名字或 <包>/<名字>，或 --all-features）照旧决定真跑测试的那几次收尾补 --test-threads、跑完核用没用上卡（gpu_test_run）。

真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡（test_runs_multi_host_configuration、gpu_test_run、gpu_card_unused_reason）：
  工作进程在源码副本里起 cargo，副本不是 git 工作树，测试照 git common-dir 找不到多机配置，crates 那侧找不到就直接报错（不再当两个开关都关）。
  所以真跑测试的每一次（基线的批、变异、插桩跑，带不带 gpu 特性都算；只编不跑的 --no-run 不算）的环境里设 SINGLEFS_MULTI_HOST_CONFIG=<多机配置的绝对路径>。
  多机配置在哪：调用方设了 SINGLEFS_MULTI_HOST_CONFIG 照它（相对路径按本进程的当前目录取成绝对路径）；没设取本模块所在仓的
  ${GATE_MUTATION_MULTI_HOST_CONFIG:-<git common-dir 的上一层>/multi-host.env}（取不到 common-dir 退回本模块所在仓的根，同 scripts/gate/admission.py 的
  multi_host_configuration_file；GATE_MUTATION_MULTI_HOST_CONFIG 是自证换它用的，只在临时服务底下认）。那份文件不在的，run_stage 起任何 cargo 之前就拒、退 2、一条都不跑（配置错，不当本次未跑）。
  变异那一条判出「没红」（verdict_of_run 交 failure）、点名的测试恰好对上一个用例而且都是 ok 的，带 gpu 特性的再看这一条用没用上卡，用不上记「本次未跑」（同文件头「本次未跑」：
  不算抓到、不算没红，逐条打出来，整道不因它判红）：
    点名的测试那一段输出（libtest 的「---- <全名> stdout ----」；输出里一段都没有时看整份）里有 CRASH_POINT_REPLAY_GIVEN_STATES 行，
       而每一行的 states_judged_by_the_gpu_here 都是 0：给定状态全交 CPU 判，GPU 一个状态都没判。
  点名的测试打了 FAILED 的照记抓到（.claude/rules/mutation-sampling.md「改了一个格式常量之后」一节的次序：FAILED 先于本次未跑；FAILED 而它自己那一段输出里、或所有段之外有行首 GPU_DEVICE_LOST 的行的除外，那一条是本次未跑），不再看用没用卡。
  不打 CRASH_POINT_REPLAY_GIVEN_STATES 行的点名测试（GPU 对拍、取卡、卡池这些不经给定状态判的），不另判用没用卡、照常判：
    要卡的用例都经 crate singlefs-verification 的 gpu_unit_checks::gpu_test_start 取卡（用例 every_gpu_test_takes_its_cards_through_gpu_test_start 判这一条），
    它不读多机配置的 GPU 开关：卡起来了就在卡上跑，本机没有 NVIDIA 卡打 NOT_RUN_NO_GPU_ADAPTER（上面「本次未跑」那一路已经认），
    有卡而一张都起不来当场 panic（测试红），没有悄悄改交 CPU 判的一支；不要卡的（源码扫描、假设备的账本单测）判的东西跟卡无关。
  弄坏开关 test-runs-configuration-unpassed、gpu-unused-as-missed、given-states-count-ignored（见下）。

开 GPU 时崩溃判段那几行补 gpu 特性（gpu_route_rows，plan_rows 里；配 GPU 必用 GPU，不降级）：
  多机配置（上一段的找法，test_runs_multi_host_configuration）写了 ENABLE_GPU=1 时，点名测试的单元闭包够得着 GPU_ROUTE_UNITS 里一处选路函数
  （crate singlefs-verification 的 crash_state_enumeration::crash_point_replay_judge_route 与 crash_amplification::judge_not_run_without_the_gpu_feature：
  配了 GPU 而二进制没编进 gpu 就判段不跑、只录就返回 ok）、cargo 参数里没开 gpu（也没 --all-features）的行，这一趟的 cargo 参数补 --features gpu
  （-p 点名了那个包写 gpu，没点名写 singlefs-verification/gpu；插在第一个 `--` 之前）。之后这一行起的每一次 cargo 都照补了之后的参数：
  真跑测试的那几次照上面一段交多机配置；键照补了之后的特性算（闭包按这一组特性收、特性进键），配置换了开关键跟着换。
  变异表里那一行原样（Row.fields 不动，用时表与索引照它取名字）；stderr 一行点名补了哪几行。闭包算不出（拒算）的行判不了够不够得着，照原样跑；
  补了之后点名的测试不编的（#[cfg(not(feature = "gpu"))] 的用例，判的就是没编 gpu 的二进制）照原样跑，stderr 逐行点名。
  弄坏开关 gpu-route-feature-unadded：不补。

点名测试够不着被改坏的函数（static-check 可达性，reachability_of_rows；空转代码要全部优化，跑不到的代码没有存在的意义，
  records/2026-11-02-一百七十六节欠账.md）：静态判，不编、不跑、不插桩（覆盖判不了：GPU 路上排计划时对基镜像恢复一次，被改坏的那几行执行过、结局不看）。
  每一行照门禁实际给它用的那一组参数与多机配置判：
  · 参数：多机配置（test_runs_multi_host_configuration 的找法）写了 ENABLE_GPU=1、点名测试的单元闭包够得着 GPU_ROUTE_UNITS 的，照 gpu_route_rows 补 gpu 特性
    （同一份 gpu_route_feature_word_of），闭包照补了之后的参数算；补了就不编的照原样参数算（门禁照原样跑，判段不跑，CPU 判器同样不跑）。
  · closure：被改坏的单元（UnitKeys.mutated_units，按种类、文件、限定名认）一个都不在这一组参数下点名测试的单元闭包里：开不开 GPU 都调不到。
    点名测试读被改坏那份文件的源码文本的（闭包里有那份文件的 data-file 单元，或罩住它的 data-directory 单元：include_str! 与运行期读文件、读目录），
    算够得着、不判这一行（gpu-route 也不判：读到的文本看得见改动，与走哪条判段无关；source_text_reader）。按整份文件算，偏宽：测试只扫文件的一段时，
    改在别的段也算够得着。
  · 闭包拒算（算不出键：runtime-path 读真仓的测试、current-exe 起自己的子进程这一类，ClosureInfo.key 是 None）：行键那一路照旧拒算、不打键行；可达性照已收的成员
    同一套判：被改坏的单元（或读到那份源码文本的数据单元）落在已收部分里算够得着，不报；没落进去的报 REACHABILITY_UNJUDGED，原因句写「闭包拒算、已收部分里没有被改坏的单元」
    （没收进来的那一段可能调得到它，所以不报够不着）。开 GPU 时截过的闭包同理：已收部分里有就照截过的闭包判，截过的已收部分里没有的同样报没判，不报 gpu-route。豁免照旧。
  · gpu-route：多机配置开了 GPU、闭包够得着 GPU_ROUTE_UNITS 的行，从同一个测试单元、同一组 cfg 与特性另收一份截过的闭包（lib_function_units.py 的
    rust_closure_of 的 stop_at，它文件头 Rust 闭包「到这些单元就停」），截在 REACHABILITY_GPU_ROUTE_STOPS：崩溃枚举与崩溃放量判段 CPU 那一臂连到 CPU 判块的
    那一条边（按边停：测试或别的函数直接调 check_blocks_where_until 一族照连），给定状态引擎的 judged_on_the_cpu_here 与排计划时那一处宽松的基镜像恢复（按单元停）。
    被改坏的单元在全闭包里、一个都不在截过的闭包里的判。两路共用的代码（观察者、搭建、建 GPU 判表、取样那一支直接调的 CPU 判器）在截过的闭包里照留。
  豁免（截过的闭包里认得出就不判 gpu-route，打 REACHABILITY_EXEMPT 行）：
  · cpu-configuration：测试自己写一份关着 GPU 的多机配置交给引擎（cpu_configuration_writer）。认法：截过的闭包里有单元的串字面量含 ENABLE_GPU=0
    （测试代码：集成测试目标里的单元、带 #[test] 或 cfg(test) 的；库代码也认：测试经库函数写只开 CPU 的配置，库函数在闭包里，例如
    crates/singlefs-verification/src/crash_library.rs 的 multi_host_configuration_on_the_cpu_keeping_the_prune_server_of），或测试代码的同一个单元里既有空串字面量、
    又有 multi-host 起头 .env 收尾的串（写一份空配置，就是单机 CPU）。库代码里只是提到 ENABLE_GPU=0 这几个字而不写配置的单元进了闭包也算，偏宽。
  · gpu-tables-refused：GPU 许建不出判表、回到 CPU 判器的镜像：截过的闭包里有 REACHABILITY_CPU_FALLBACK_ROOTS 里的函数（cpu_fallback_root_in）。
  起跑先核 GPU_ROUTE_UNITS、REACHABILITY_GPU_ROUTE_STOPS、REACHABILITY_CPU_FALLBACK_ROOTS 里每个函数在 <根> 的源码里都在（reachability_names_missing），有的不在报配置错、退 2；
  一个都不在的（门禁样本那种小根，没有崩溃放量那一族函数）不是配置错：打 REACHABILITY_NOT_JUDGED 一行写原因，一行都不判，退出码不受它影响（reachability_of_rows 抛 ReachabilityNotJudged）。
  static-check 一律判可达性（没有开关参数）；门禁 code-source-discipline 的 mutation-tables 格与 scripts/gate/apply-writer-patch.py 的变异锚点静态核都经它。
  判不了的行逐行打 REACHABILITY_UNJUDGED：锚点失效、--manifest-path 写坏、被改坏的单元或点名测试拒算、闭包算不成、闭包拒算而已收部分里没有被改坏的单元
  （已收部分里有的照判，不在此列）、闭包拒算而截过的闭包已收部分里没有被改坏的单元、截过的闭包算不成。
  已知会漏：闭包同名就连、偏宽，被改坏的函数经同名的别处连进截过的闭包就不判；GPU 路上将来再出一处调 CPU 判器而结局不看的，要补进截断表。
  点名测试的闭包照 KeyContext 那一份算法并行算（key_workers；precompute_closures_from_shared_parent：fork 之前共用词法单元、先算一条、冻进永久代），
  截过的闭包另开同样多的 fork 子进程并行算（reachability_cut_answers，fork 之前同样冻）；截过的闭包不算键、不进判定库。
  内存：几行点名同一份闭包的排在一起判，成员集合（种类、文件、限定名）每份闭包只摊开一次、这一组判完就放（reachability_plans_by_closure），不给几千份闭包同时各留一份。
  弄坏开关 reachability-stops-ignored、reachability-edges-as-units、reachability-closure-unchecked、reachability-cpu-configuration-ignored、
  reachability-cpu-configuration-test-code-only、reachability-fallback-roots-ignored、reachability-stops-unverified、reachability-source-text-ignored、reachability-refused-unjudged（弄坏开关那一节）。

带 gpu 特性的格测试参数收尾带 --test-threads（with_gpu_test_threads，run_cargo_command 起命令前调；.claude/rules/verification.md 里 GPU 那一条：
  控制测试线程的只有变异那一路，单个 GPU 测试二进制带 --test-threads 1 跑；放开名单里的测试目标放开）：
  这一次起的 cargo test 是 gpu_test_run 认的那一种（`--` 之前开了上卡的特性、真跑测试）时，libtest 参数收尾带 --test-threads：
    · 基线的批、变异、记覆盖的插桩跑同样照办（都经 run_cargo_command）；只编不跑的几次（--no-run）与不带 gpu 特性的不带；
    · cargo 参数点名的测试目标（--test <名>，可以几个）全在放开名单里的，
      带每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads：SINGLEFS_THREAD_CAP 设了照它，否则 .claude/heavy-thread-default）；
      别的（--lib、没点名目标、有一个目标不在名单里）带 1；登记取不出的打一行说明、照旧带 1；
    · 放开名单（gpu_released_test_targets，这份名单只有变异那一路用，init_worker 照被判的根交给 with_gpu_test_threads）= 本模块的 GPU_TEST_THREADS_RELEASED_TARGETS
      加验证包登记的只经卡池取卡的测试目标（card_pool_only_test_targets：被判那棵树的
      crates/singlefs-verification/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs 里 FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL
      每一项 "tests/<名>.rs" 的 <名>；那份文件不在的树不读出，读不了或认不出那张表的打一行说明、这一次只放开 GPU_TEST_THREADS_RELEASED_TARGETS）。
      放开的判据：一个测试二进制里并行的 GPU 用例要卡都经这个进程的卡池按显存排队（crate singlefs-verification 的 gpu_test_start::GpuCardPool，
      借不到时 CardLeaseWaitingRoom 等别的线程还卡），不各自起设备、各按空闲显存开草稿区互抢；变异那一路上卡的格一次只有一个测试二进制在卡上（文件头「上卡的格一格接一格」），
      卡池的显存账按进程记，同一个进程里排队就够。名单里的文件由验证包那份用例逐个核「只经 gpu_card_pool_or_report_not_run 取卡、不拿第一张能用的卡、不拿整份卡表、
      不直接建判器」（every_test_file_moved_onto_the_card_pool_takes_its_cards_only_from_the_pool），挪进挪出都由那份名单定，这里不另抄一份。
      GPU_TEST_THREADS_RELEASED_TARGETS 里的 GPU 对拍目标多数用例走卡池，量显存那几条照整张卡起（那份用例的 TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL 登记了它），按显存排队的放开照旧。
      不放开、带 1 的：验证包的库内单测（--lib：卡池与账本自己的单测在里面，那份扫描用例不扫 gpu_unit_checks 自己，库内单测没有只经卡池的登记）、产品 GPU 包带 tests-on-a-real-gpu 的目标
      （经 singlefs-background-gpu 自己的取卡与起设备，不走验证包的卡池）、验证包里没登记进那张表的测试目标（取卡可能经库代码绕开卡池，那份用例扫不到）；
      要放开它们，先让它们只经卡池取卡、登进那张表。
      弄坏开关 gpu-test-threads-all-one：一个都不放开，带 gpu 特性的格一律带 1；
    · 行里 `--` 之后自己已写 --test-threads（--test-threads=N 或分开写）的原样不动、不重复补，行写几就是几（行的作者定了，不按名单改写）；
      与 --exact 并存时各只出现一次（--test-threads 补在末尾，在点名用例之后）；没有 `--` 的参数先补 `--`。
  线程参数不进行键（文件头「行键」那一份）：行里自己写的 --test-threads=N 是 libtest 开关、照 lib_mutation_units.py 的 parse_cargo_arguments 进键（libtest:--test-threads=N）；
  这里补的不改行的六段，也不进键——它由键里已有的两样（带不带 gpu 特性、测试目标）加一个每台的常数（登记的线程数）决定，没有新的输入；线程数只管用例并行多少、
  放开的目标按显存额度排队，与 -j / --jobs 同属不改判定的调度参数；进键则登记值一改（每台默认重活线程数是登记的值），所有带 gpu 的行已入库的判定无故作废、全部现判。
  弄坏开关 gpu-test-threads-unadded：不补，带 gpu 特性的格多个 GPU 对拍用例并行抢显存。

重测试目标只许 release（release_forced_row、release_unwritten_rows；.claude/rules/test-script-discipline.md「编译与算法都要快」第一句：穷举、长时间随机、真设备这类用例用优化构建跑）：
  判法只在 scripts/mutation/lib_release_only_tests.py（它文件头写全了怎么算命中、怎么算已带 release），这里不另写：命中 = cargo 参数选中了登记的崩溃枚举用例目标
  （.claude/gate.d/stage-inputs.tsv 的 crash-case: 行 test= 点到的包与测试目标），或特性里带 gpu（gpu、<包>/gpu、tests-on-a-real-gpu、--all-features）。
  登记表：static-check 读被判那棵树 <根> 里的（同拷贝范围，judged_registry_root；<根> 里没有那张表打 ERROR 退 2，不回落）；run_stage、plan、row-plan、--list-items、--item
  读本模块所在那一份仓的（RELEASE_REGISTRY_ROOT：判别力样本的目录里没有登记表；自证里要另一份的那一格换成自己造的一份）。
  · 门禁（plan_rows：run_stage、plan、row-plan、--list-items 同一份）：命中而没带 release 的行，这一趟的 cargo 参数补 --release（插在第一个 `--` 之前；
    行里已带 --release 或 --profile release 的不重复加）；之后这一行起的每一次 cargo（基线的批、变异的先编与跑、插桩编译与插桩跑、先编一份依赖）都照补了之后的参数。
    开 GPU 时补了 gpu 特性的行（gpu_route_rows）补完特性再照判一次。变异表那一行原样（Row.fields 不动，用时表与索引照它取名字）；
    stderr 一行报补了几行，点名前 REASONS_SHOWN 行与依据。
  · 档进键：--release 是 cargo 参数里的开关，照 lib_mutation_units.py 的 parse_cargo_arguments 进行键的「开关与特性」，闭包照 release 档的 cfg 收（不带 debug_assertions）；
    release 档的调试信息与增量照「调试信息与 release 增量」那一段设、取值不进行键。补了之后的键与行里自己写 --release 的键相同，与不带 release 的键不同：头一趟这几行现判，不迁移旧键。
  · static-check：命中而第五段没写 release 的行逐行打 RELEASE（文件头 static-check 那一段），退 1；crates-mutation-replay 派活之前不判（那边照上一条补 --release 起）。
  登记表不在、读不了、crash-case: 行写法认不出：run_stage 一条都不跑（退 1），plan、row-plan、--list-items 照算不出键拒，static-check 打 ERROR 退 2；不当不命中。
  弄坏开关 release-unforced、release-unchecked（弄坏开关那一节末尾「重测试目标只许 release」那两行）。

验证包的测试目标不编装置二进制（default_features_off_row，plan_rows 里，补 --release 与补 gpu 特性之前）：
  cargo 给集成测试编目标时把本包的二进制目标全编一遍；验证包 src/bin/ 下的装置二进制各挂缺省开着的特性 apparatus-binaries（crates/singlefs-verification/Cargo.toml），
  起法带 --no-default-features 就一个都不编（源码里没有一处读这个特性）。
  · 命中：cargo 参数（`--` 之前）-p 只点名 singlefs-verification、点名了 --test <名>（可以几个）、没有别的选目标的选项（--lib、--bin、--bins、--tests、--all-targets 等，
    选项表同 lib_release_only_tests.py 的 TARGET_OPTIONS）、自己没写 --no-default-features：这一趟的 cargo 参数补 --no-default-features（插在第一个 `--` 之前）。
    带 --lib、--bin、--bins、--tests 的、没点名目标的（cargo 本来就不为它编二进制，补了反而让库在两种特性集下各编一份）、点名别的包或几个包的、没点名包的一个字不变。
  · 之后这一行起的每一次 cargo（基线的批、变异的先编与跑、插桩编译与插桩跑、先编一份依赖）都照补了之后的参数；变异表那一行原样（Row.fields 不动）。
  · 进键：--no-default-features 照 lib_mutation_units.py 的 parse_cargo_arguments 进行键的「开关与特性」（行键、基线键、覆盖号都带），补了之后的键与行里自己写 --no-default-features 的键相同，
    与不补的不同：命中的行头一趟现判，不迁移旧键。与 --features gpu 同给时（-p singlefs-verification --release --features gpu --test <名> --no-default-features）gpu 特性照开，
    缺省特性里只有 apparatus-binaries，关掉它不影响 gpu 的依赖。开 GPU 时补 gpu 特性（gpu_route_rows）与补 --release 都建在补过的参数上。
  · static-check 的可达性（reachability_of_rows）按第五段原样的参数收闭包，不补：没有一处源码读 apparatus-binaries，闭包不因它变。
  弄坏开关 no-default-features-unadded：不补（弄坏开关那一节「重测试目标只许 release」那两行之后一行）。


--selftest 的挑格与并行：自证的格分在 selftest_cell_function_names 列的格函数里（一个格函数里的格互相有次序，不拆开），挑格与并行都以格函数为单位。
  · 挑格：--cells 点名的格函数只走点名的；没给 --cells 而打开了弄坏开关，只走 selftest_break_cell_table 里这些开关点名的格函数（并集，加核表那一格 selftest_break_cell_table_cell）：
    点名的格函数里有格判错就停在那里；一格都没判错再把其余的格函数走完，走完有判错的在第一行写「点名的格函数一格都没红」，要改表里这几条开关的行。
    表里值是 None 的开关（没有哪一格点名它）走全部；空元组的开关（由别的自证证红）只走核表那一格；没有开关走全部。
    核表那一格判：表点名的格函数都存在、本文件代码里读的每一条弄坏开关表里都有一行、表里的开关只走点名的格函数。
  · 并行：格函数同时跑的个数 --jobs 给了照它，不然取两者的小者（selftest_cell_jobs_for，开跑时打一行依据）：每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads，
    取不到打一行说明、一个一个跑）、这一台能给的内存（host_memory_capacity：多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项；取不到时用这一条拿到的上限
    task_memory_cap_bytes，再没有这一项不收）容得下的个数
    (内存 − base_bytes) ÷ extra_bytes（每多开一个子进程再占 512 MiB：按整趟的 cgroup 峰值摊到每个子进程，不按子进程的 ru_maxrss 取——fork 出的子进程
    与父进程共用的页照算在 ru_maxrss 里，按它取会比真占的大一倍上下）。不拿服务给的上限除：服务从小上限起步、撞了才放大，开得少量到的峰值就小，下一趟给的还是小；
    开多了撞上限由服务放大重排；
    依据与量法见 records/2026-11-08-重型门禁执行问题与缺陷总结.md「卡与核利用率」；
    1 或只有一个格函数在本进程里按次序走；不然每个格函数 fork 一个子进程，各用自己的 Selftest 与 <临时目录>/cell-<格函数名>/（假 cargo 记录、编译目录、运行记录互不踩），
    全部格函数进同一个派活池，「要等对方起来、机器忙时会记本次未跑」的几个（selftest_timing_sensitive_cell_functions）排在最前、与别的格函数同时跑
    （同时跑的个数不超过线程数，它们等的是 2 秒的假 cargo；真被挤成本次未跑的照记、整份退 77 不判绿）；
    结果（格数、判错的格、本次未跑的格）按格函数的登记次序并回来；收回的个数与派出的对不上（缺的、多的逐个点名）整份判红。
    格函数里的格互相有次序（同一份小仓上头一趟、第二趟复用这一类），一个格函数内部不并行；单个最久的格函数（selftest_unit_key_cells、selftest_reuse_fix_cells）是派活池收尾的下限。
  · 子进程与收尾（selftest_cell_child、selftest_fork_cell_functions）：子进程自己换成新会话与进程组的头、SIGTERM 取默认处理（父进程为收尾装的处理器带进格函数，
    格函数里的进程池 terminate 工作进程时，工作进程退不出处理器，进程池收尾卡死）；父进程按 selftest_cell_dispatch_order 派：要等对方起来的先，再久的（selftest_heaviest_cell_functions）；
    每个格函数跑完核它那一组里没有留下进程，留下的停掉（先 TERM 再 KILL）、写成一格判错点名每个进程；
    每个格函数至多跑 function_seconds（600）秒，超了先记下组里每个进程的状态、等在内核哪里与命令行，再停整组，记这个格函数没跑成并带上那份现场，父进程不无限等；
    每个格函数跑完往 stderr 打一行：用时、CPU 秒数、峰值内存（ru_maxrss，含它等过的孙进程）。派出的子进程没留下结果记一格判错。
  · 调度本身的格：selftest_cell_scheduling_cells（默认并行度的几个取值、要等对方起来的与久的先、两个探针 jobs 给 2 时同时跑、派几个收几个、子进程的进程组与 SIGTERM、
    留下的进程、超时，探针真起子进程）。

自证夹具（load_selftest_fixtures、SELFTEST_FIXTURES_PATH）：只被自证与弄坏开关的自证读的模块级常量（样本 crate 的源码、样本变异表的行，
  名字以 SELFTEST_ 或 Selftest 起头的那一批）放在同目录 crates_mutation_rows_selftest_fixtures.py，run_selftest 开跑时按路径载入一次、照它的 FIXTURE_VALUES 放进本模块
  （fork 出来的格函数子进程照带；按路径重新载入本模块的子进程自己再调一次 load_selftest_fixtures）。运行期也读的（FAKE_CARGO 给 sample-tools 子命令，FAKE_LLVM_TOOL 给 mutation-shard-run.sh）与名字不带这两个起头的留在本文件。
  自证起子进程用 [sys.executable, "-c", <名字>, …] 跑的代码串（POOL_ERROR_CHILD_PROGRAM、STAGE_CHILD_PROGRAM）也留在本文件、模块级只赋一次：
  scripts/gate/lib_function_units.py 的 fold 折不出经 globals() 放进来的名字，起这个子进程的格会整行拒算 inline-script-not-literal，break-switch-replay 格就没有键。
  def 头的缺省值在 import 时求值，不写夹具里的名字（build_selftest_repository 的 source 缺省取 None）。

弄坏开关（GATE_MUTATION_BREAK，只给证红用，逗号分隔可给几个；CRATES_MUTATION_ROWS_BREAK 是同义名，直接起本模块自证时用，两个变量都认）：
  selftest-break-cells-ignored  按开关挑格时一律走全部格函数
                       （本模块自证「表里有的弄坏开关只走它点名的格函数（加核表那一格）、次序照格函数的次序」那一格必须判错）
  selftest-break-table-entry-dropped  弄坏开关表少一行 nobaseline
                       （本模块自证「本文件代码里读的每一条弄坏开关在弄坏开关表里都有一行」「表里有的弄坏开关只走它点名的格函数…」两格必须判错）
  cell-jobs-by-threads-only  格函数同时跑的个数只看线程数，不收内存上限
                       （本模块自证「默认并行度：线程数与内存上限容得下的个数两者的小者（31 线程无上限 31、上限 11420M 19、上限 4000M 5、上限 100M 1、4 线程 4）」那一格必须判错）
  cell-jobs-by-task-cap  格函数同时跑的个数按这一条拿到的上限（task_memory_cap_bytes）除，不按这一台能给的内存除
                       （本模块自证「默认并行度按这一台能给的内存除：…」那一格必须判错）
  cell-heaviest-first-ignored  派格函数时不把要等对方起来的与久的排前面，照格函数名单的次序
                       （本模块自证「派出次序：要等对方起来的先，再久的（照先派表的次序），别的保持原次序」那一格必须判错）
  cell-outcome-dropped  派出去的格函数里，照登记次序排在最后的那一个的结果收回时丢掉（像子进程的结果没收回来）
                       （本模块自证「派几个收几个：两个探针派出去、收回两个结果」与并回来时「派出去的格函数都收回了结果（个数与名字都对得上）」两格必须判错；
                        丢的正是留下进程、超时那两个探针的结果，「格函数跑完它那一组里留下的进程被停掉，写成一格判错」「格函数超过用时上限：…」两格跟着判错）
  cell-child-inherits-parent-process-state  格函数子进程不换进程组、不还原 SIGTERM（带着父进程为收尾装的处理器）
                       （本模块自证「格函数子进程：SIGTERM 取默认处理、自己是新进程组的头（进程池 terminate 它的工作进程时不卡死）」
                        「格函数跑完它那一组里留下的进程被停掉，写成一格判错」「格函数超过用时上限：停整组、写明停时组里每个进程在等什么，不无限等」三格必须判错）
  cell-leftover-processes-kept  格函数跑完不核它那一组里有没有留下进程
                       （本模块自证「格函数跑完它那一组里留下的进程被停掉，写成一格判错」那一格必须判错）
  cell-limit-ignored   不看每个格函数的用时上限
                       （本模块自证「格函数超过用时上限：停整组、写明停时组里每个进程在等什么，不无限等」那一格必须判错）
  nobaseline           跳过基线：点名的测试在没改坏的副本上本来就红的行照样记抓到（crates-mutation-replay 红样本、本模块自证都必须判错）
  timingsignored       派活不看用时表（自证「上一次跑得久的先派」那一格必须判错）
  dispatch-by-crate-only  派变异只按 crate 归堆，不先按编译配置、不按改的文件排（rows_grouped_by_crate）
                       （本模块自证「派变异的次序：六行（三个编译配置…）先按编译配置归堆…」「上一次用时不短于门槛的行先派…」两格必须判错）
  target-measured-every-cell  每格判完都量一次编译目录占盘，不隔几格量
                       （本模块自证「编译目录占盘不是每格都量：…」那一格必须判错）
  worker-peaks-unkeyed-by-target  每个工作进程的峰值只按包记、按包估，不记也不看测试目标那一级
                       （本模块自证 selftest_peak_target_cells 里「这一趟要判的行估峰值取哪几个记录名」「估值：light 的行取它自己那一级的 5 GiB…」「判一格期间采样线程要记的范围…」三格必须判错）
  worker-jobs-held-after-memory-cut  内存把工作进程数减下来之后编译并行度一律留在减之前那一档，不照减之后的个数重取
                       （本模块自证「内存把工作进程从 4 个减到 1 个：编译并行度照 1 个重取成 …」那一格必须判错）
  unfiltered-rows-unchecked  static-check 不判「第五段 `--` 之后没有测试名筛选词」
                       （本模块自证「static-check 判「第五段 `--` 之后没有测试名筛选词」…」那一格必须判错；门禁 code-source-discipline 的样本 mutation-tables-unfiltered-red 判绿）
  workers-by-processor-half  没给 GATE_MUTATION_WORKERS 时工作进程数取核数一半、至多 16、编译并行度按核数 ÷ 工作进程数，不取每台默认重活线程数、取不到也不拒
                       （本模块自证「没给 GATE_MUTATION_WORKERS：工作进程数取每台默认重活线程数…」「…取不到每台默认重活线程数（SINGLEFS_THREAD_CAP=abc）…」
                       「编译并行度的整台线程数取每台默认重活线程数…」「先编那一份的并行度：…」四格必须判错）
  workers-beyond-rows  工作进程数不收「这一趟要现跑的条数」那一项（文件头「工作进程数」），行少而线程数、上限大时开得比要判的行多
                       （本模块自证「工作进程数取 GATE_MUTATION_WORKERS 与要现跑的条数的小者，没给内存项时头一行不提内存（20 条开 16 个、5 条开 5 个）」必须判错）
  memory-cap-ignored   开几个工作进程不看内存上限（TASK_SERVER_MEMORY_CAP_BYTES、自己 scope 的 memory.max），照线程数开满
                       （本模块自证 selftest_memory_cells 里除「内存上限的常量与大小写法：…」那一格之外的二十一格全部必须判错，头一格是「服务交的内存上限 20 GiB、每个工作进程的峰值 4 GiB（编译并行度 4 下的实测）…」，
                       其中含「内存上限装不下一个工作进程的估值：run_stage 照开 1 个起跑…」，也含「照这一次服务给的上限开工作进程（上限只够 1 个工作进程）：…」「照这一次服务给的上限开工作进程（上限够 3 个工作进程）：…」
                       「照这一次服务给的上限开工作进程（1 个也不够）：…」三格，还含「照这一次服务给的上限开工作进程（够开满）：…」：它开的个数照样是 4 个，判错在头一行没有「这一台能给的内存」那一项（不看内存上限就不按那一台的内存除）；
                       selftest_worker_peak_cells 里经 memory_bound_for、memory_bound_probing 的十三格也判错：「固定编译并行度：…」「编译并行度跟着进程数走：…」「从没量过：…」
                       「只有更低编译并行度的实测：…」「峰值记录里有一份内容不是正整数：…」「峰值记录的 worker-peaks 是一份文件、不是目录：…」「交给 run_stage：…开 2 个工作进程…」
                       「只判小包的行：…」「同样的记录同时判大包与小包的行：…」「要判的包里有一个从没量过：…」「交给 run_stage 只判小包（tiny）的行：…」
                       「不写峰值记录的一趟（门禁样本根）：…」「交给 run_stage、这几行点名的包从没量过：…」）
  memory-cap-short-self-exit  内存上限装不下一个工作进程的估值时派活之前自己按撞内存上限退 250、不开工，不照开 1 个起跑（文件头「内存上限」的「撞内存上限」）
                       （本模块自证 selftest_memory_cells 里「内存上限装不下一个工作进程的估值：run_stage 照开 1 个起跑…」
                       「照这一次服务给的上限开工作进程（1 个也不够）：…」两格必须判错，别的格不红）
  memory-cap-exits-before-fewer-workers  这一条拿到的上限装不下要开的个数时不往下找装得下的个数，照一个都装不下办（开 1 个起跑）（文件头「内存上限」的「撞内存上限」）
                       （本模块自证 selftest_memory_cells 里「照这一次服务给的上限开工作进程（上限只够 1 个工作进程）：…」「照这一次服务给的上限开工作进程（上限够 3 个工作进程）：…」
                       两格必须判错，别的格不红）
  unmeasured-target-borrows-package-peak  测试目标那一级自己没有实测时借它的包（整个包最新几份里最大的），不取同包测试目标那一级的中位数、也不取起步值（文件头「内存上限」的「怎么除」）
                       （本模块自证 selftest_peak_target_cells 里「没有实测的测试目标取同包测试目标那一级实测的中位数：…」「包里一份测试目标那一级的实测都没有：…取起步值…」两格必须判错，别的格不红）
  workers-not-grown-after-measuring  跑着不按实测加开工作进程：开跑时定的个数一直开到底（文件头「内存上限」的「跑着加开」）
                       （本模块自证 selftest_worker_growth_cell 里「跑着加开：…」那一格必须判错）
  memory-max-not-hard-cap  内存上限取到服务交的变量或多机配置的值就不看自己 scope 的 memory.max：scope 真实上限只有 20 GiB，多机配置写 50G 就按 50G 开
                       （本模块自证 selftest_memory_cells 里「多机配置 50G、scope 的 memory.max 20 GiB」「服务交的变量 50 GiB、scope 的 memory.max 20 GiB」两格与
                       「run_stage 整趟：服务交的变量 50 GiB、scope 的 memory.max 8 GiB」一格必须判错，别的格不红）
  task-memory-constants-skewed  内存上限常量的核对拿 CGROUP_ROOT 接上一段多余的路径再比 scripts/hosts/task_memory.py 的同名常量
                       （本模块自证 selftest_memory_cells 里「内存上限的常量与大小写法：本文件留着的模块级赋值与 scripts/hosts/task_memory.py 里用的同一个值」那一格必须判错，别的格不红）
  records-written-at-end  判定库记录（基线、变异）先攒在工作进程内存里，进程池正常收尾时才一并写（被怀疑过的样子：整组被杀、判过的格不留下），init_worker 换上 DeferredWriteLibrary
                       （本模块自证 selftest_killed_run_resume_cell「一趟跑到一半整组硬杀，重跑同一条命令」那一格必须判错，别的格不红）
  worker-peaks-unkeyed-by-package  读每个工作进程的峰值记录时不分包，每个包都读成所有包合起来的：同一个编译并行度下取各包里最大的（大包的历史峰值判死只判小包的一趟）
                       （本模块自证「只判小包的行：…」「同样的记录同时判大包与小包的行：…」「要判的包里有一个从没量过：…」「探针量到之后放开：…」
                       「交给 run_stage：GATE_MUTATION_CARGO_JOBS=3、…开 2 个工作进程…」「交给 run_stage 只判小包（tiny）的行：…」六格必须判错，别的格不红；
                       「交给 run_stage：…开 2 个工作进程…」判错在包 tiny 与它的测试目标那一级 tiny--lib 各记了一份同样的读数，不分包时合起来读成「最新 2 份」、不是「最新 1 份」）
  worker-peaks-unkeyed-by-jobs  读每个工作进程的峰值记录时不分编译并行度，每个包里各编译并行度都取全部记录里最大的（量与除不认编译并行度的样子）
                       （本模块自证「记下的峰值按包、按编译并行度分开、读得回来：…」「固定编译并行度：…」「编译并行度跟着进程数走：…」三格必须判错）
  worker-peaks-window-unbounded  读峰值记录时每个包、每个编译并行度取全部记录的最大、不只看最新的 WORKER_PEAKS_RECENT 份，也不删更旧的
                       （本模块自证「每个包、每个编译并行度只看最新的 256 份：…」那一格必须判错）
  jobs-feedback-ignored  试每个进程数时编译并行度都照天花板那个进程数算，不照这个进程数算：开的进程少了、编译并行度跟着变大，峰值没跟着算
                       （本模块自证「编译并行度跟着进程数走：…」那一格必须判错）
  unmeasured-worker-peak-guessed  某个包在某个编译并行度与更高的都没有实测时，每个工作进程的峰值照写死的默认 3G，不取保守值（规划与探针派活都这样取）
                       （本模块自证「不写峰值记录的一趟（门禁样本根）：…」「只有更低编译并行度的实测：…」「探针派活：…」「交给 run_stage、这几行点名的包从没量过：…」四格必须判错）
  new-package-run-single-worker  要判的包里有从没量过的（峰值记录里这个包一份都没有），规划时照保守值（内存上限本身）进这一项，整趟只开 1 个工作进程，不靠探针
                       （本模块自证「从没量过：…」「要判的包里有一个从没量过：…」「交给 run_stage、这几行点名的包从没量过：…」三格必须判错）
  unmeasured-package-not-probed-alone  派活时没有实测的包不单独跑：在跑的格与要派的格点名的包里，在这一趟的编译并行度下没有实测的那几个不算进峰值，照样同时派
                       （本模块自证「探针派活：…」「交给 run_stage、这几行点名的包从没量过：…」两格必须判错）
  unreadable-worker-peaks-guessed  峰值记录读不了时每个工作进程的峰值照写死的默认 3G，不取保守值
                       （本模块自证「峰值记录里有一份内容不是正整数：…」「峰值记录的 worker-peaks 是一份文件、不是目录：…」两格必须判错）
  worker-tree-self-only  量一个工作进程的用量时只算它自己，不算它起的整棵进程树
                       （本模块自证「量一棵进程树：…」那一格必须判错）
  worker-peaks-unrecorded  工作进程起来时不起采样线程，一份峰值记录都不写
                       （本模块自证「交给 run_stage 的工作进程各写一份峰值记录：…」「交给 run_stage、这几行点名的包从没量过：…」两格必须判错：后一格量不到，探针之后放不开）
  sample-root-peaks-recorded  被判的不是本仓（门禁样本根）、也不在临时 prune-server 底下时，工作进程照样写峰值记录（样本 crate 的小读数掺进本仓那一份的样子）
                       （本模块自证「样本根上的工作进程不写峰值记录：…」那一格必须判错）
  multiple-packages-last-only  cargo 参数里有多个 -p 只认最后一个（lib_mutation_units.py 的 parse_cargo_arguments）
                       （本模块自证「多个 -p：…」那一格（selftest_multiple_packages_cell 调 lib_mutation_units.py 的 multiple_packages_cell）必须判错）
  rowplanselectionignored  row-plan 不看 --rows，照整张表算键（mutation-shard-run.sh --selftest 里 row-plan 点名两行那一格设它起 row-plan，必须判错）
  unsharded-rows-judged  设了 GATE_MUTATION_SHARDED_ONLY=1 也照现判库里没有键相同记录的行（门禁不分片、单机整张判的样子）
                       （mutation-shard-run.sh --selftest 里「没经分片判过的行，门禁不现判」那一格必须判错）
  narrowed-verdicts-recorded  缩过域的一趟照常写判定库（判定、基线、覆盖、索引都写），也照记覆盖（文件头「缩过域的一趟」）
                       （本模块自证 selftest_crash_domain_narrowing_cells 里「设了缩域变量：现判的行不入库…」那一格必须判错，别的格不红）
  narrowed-sentinel-drawn  缩过域的一趟照常抽复用哨兵、强制现判一条
                       （本模块自证 selftest_crash_domain_narrowing_cells 里「缩过域的一趟照复用库里全域下判的抓到、不抽复用哨兵…」那一格必须判错，别的格不红）
  narrowed-sharding-allowed  缩过域的一趟照常进服务分片：row-plan 照列、分片的各片与合并那一步的 run_stage 照判
                       （本模块自证 selftest_crash_domain_narrowing_cells 里「缩过域不分片…」那一格必须判错，别的格不红）
  mutated-unit-ignored  行键不带被改坏的单元指纹（自证「改了被改坏、点名测试够不着的函数：只现判那一行」那一格必须判错）
  mutated-test-service-code-keyed  被改坏的单元落在测试服务代码里也把指纹放进行键（KeyContext.components 的 mutated 一组收它）；scripts/mutation/lib_mutation_units.py 里同一条开关
                       （同样从 GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK 认）：mutate_plan 取被改坏的单元的闭包键带 entry 行，入口自己的指纹进键
                       （本模块自证「测试服务代码：被改坏的函数在测试服务代码里，改它之后一行都不现判」那一格与「mutate.sh 的键：被改坏的单元在测试服务代码里，改它之后那一行的键不变」那一格
                       （selftest_mutate_plan_test_service_code_cell 调 lib_mutation_units.py 的 mutate_plan_test_service_code_cell）必须判错）
  row-key-collision-unrefused  表里两行算出同一把行键时不判撞键，两行共用一把键、互相复用
                       （本模块自证「撞键拒算：left.rs 与 right.rs 里内容相同的函数各改一处…」那一格必须判错）
  not-run-test-name-only  本次未跑只认 test= 对得上点名的测试名（只比 :: 末段）的那一行：test=crash_state_enumeration::<项名> 记成没红
                       （本模块自证「本次未跑认项名：…」那一格必须判错）
  not-run-line-start-only  本次未跑的记号只认行首（行首空白不算）的那一处：行中间、带颜色码、排在同一行别的记号后面的都不认，点名的测试 ok 照记没红
                       （本模块自证「本次未跑的记号不在行首也认：…」那一格必须判错）
  gpu-route-feature-unadded  多机配置开了 GPU 时，点名测试走崩溃判段、不带 gpu 特性的行不补 gpu 特性：判段不跑、只录就返回 ok
                       （本模块自证「开 GPU 时点名测试走崩溃判段的那一行补 gpu 特性：…」那一格必须判错）
  gpu-test-threads-unadded  带 gpu 特性、真跑测试的 cargo test 的 libtest 参数收尾不带 --test-threads
  gpu-test-threads-all-one  放开名单一个都不放开，带 gpu 特性、真跑测试的 cargo test 一律收尾带 --test-threads 1
                       （本模块自证「只经卡池取卡的测试目标放开：…」与「带 gpu 的行收尾带 --test-threads 1、放开名单里的目标带登记值…」两格必须判错）
  reachability-stops-ignored  static-check 可达性截闭包时不截（stop_at 给空）
                       （本模块自证「static-check 可达性的 gpu-route：…」那一格必须判错）
  reachability-edges-as-units  static-check 可达性截断表里按边停的也按单元停（测试直接调的 CPU 判器入口也截掉）
                       （本模块自证「static-check 可达性放行够得着的：…」那一格必须判错）
  reachability-closure-unchecked  static-check 可达性不判 closure 那一路（被改坏的单元根本不在点名测试的闭包里也不报）
                       （本模块自证「static-check 可达性的 closure：…」那一格必须判错）
  reachability-cpu-configuration-ignored  static-check 可达性不认测试自己写的 CPU 配置
                       （本模块自证「static-check 可达性的 cpu-configuration 豁免：…」那一格必须判错，库函数那一格也判错）
  reachability-cpu-configuration-test-code-only  static-check 可达性只认测试代码里的 ENABLE_GPU=0，库代码里写的不认（测试经库函数写 CPU 配置的行报 gpu-route）
                       （本模块自证「static-check 可达性的 cpu-configuration 豁免，库函数写配置：…」那一格必须判错，别的格不红）
  reachability-fallback-roots-ignored  static-check 可达性不认 GPU 许建不出判表、回到 CPU 判器的那个函数
                       （本模块自证「static-check 可达性的 gpu-tables-refused 豁免：…」那一格必须判错）
  reachability-stops-unverified  static-check 可达性起跑不核选路表、截断表、豁免表里的函数在不在源码里
                       （本模块自证「static-check 可达性起跑先核截断表：…」那一格必须判错）
  reachability-source-text-ignored  static-check 可达性不认读源码文本的点名测试：只拿函数单元比，闭包里有那份文件的数据单元照报 closure
                       （本模块自证「static-check 可达性放行读源码文本的：…」那一格必须判错）
  reachability-refused-unjudged  static-check 可达性遇到闭包拒算的行一律报没判，不看已收的成员
                       （本模块自证「static-check 可达性的闭包拒算：点名测试的闭包拒算…」那一格必须判错；同一开关下后面三格也红）
                       （本模块自证「带 gpu 的行收尾带 --test-threads 1、放开名单里的目标带登记值、不带 gpu 的行不带…」那一格必须判错）
  baseline-key-without-target  基线键与覆盖号不带点名测试所在的包与测试目标
                       （本模块自证「基线键与覆盖号带包与测试目标：…两行的基线键、覆盖号都不同」那一格必须判错）
  baseline-batch-composition  一批基线照整批的输出判每一行：同一行单跑与整张表判出不同的基线结局
                       （本模块自证「一行的基线结局不随这一批拼进了哪几行而变：…」那一格必须判错）
  red-not-recorded     judge_row 只记「抓到」，没红、无效这些不写，同一把行键原有的「抓到」留着照复用
                       （本模块自证「记红撤绿：先抓到入库，同一把键改判没红之后，下一趟这几行现判…」那一格必须判错）
  uncounted-overwrites-counted  不算数的结局（VERDICTS_UNCOUNTED）照行键写、盖掉同一把行键原有的抓到、没红、无效
                       （本模块自证「不算数的结局不盖算数的：库里一行抓到，同一把键经哨兵强制现判出本次未跑之后…」那一格必须判错）
  baseline-red-keeps-caught  基线不绿的行不照行键记「基线不绿」，同一把行键原有的「抓到」留着照复用
                       （本模块自证「记红撤绿：先抓到入库，同一把键判一趟基线不绿之后，下一趟这几行现判…」那一格必须判错）
  library-outside-git-tolerated  被判的树取不到 git common-dir 时拿被判的根当库根往下跑（没处放也照跑）
                       （本模块自证「库根：被判的树不是 git 工作树：退 1、点名它、一条都不跑」那一格必须判错）
  run-overrides-honored-outside-service  不在临时 prune-server 底下设了 GATE_MUTATION_RUN_RECORDS_DIRECTORY、GATE_MUTATION_MULTI_HOST_CONFIG 也照旧认
                       （本模块自证「不在临时 prune-server 底下设了只给自证换的运行记录、多机配置两个变量：…」那一格必须判错）
  mutation-run-on-real-service  起 cargo test 真跑测试的那一次不起自己的临时 prune-server，照调用方的环境连那一份服务：变异版判出的崩溃结论写进真库
                       （本模块自证「变异复跑各对自己的临时 prune-server 跑：…」那一格必须判错）
  baseline-on-temporary-service  基线也起自己的临时 prune-server，不连调用方的那一份判定库服务：判过的块读不到、判完的写不进那一份
                       （本模块自证「基线连调用方的那一份判定库服务：…」那一格必须判错）
  baseline-release-incremental-kept  基线起的 cargo 也带门禁加的 CARGO_PROFILE_RELEASE_INCREMENTAL：崩溃判定的版本（codegen-units 生效值 256）与门禁 crash-point-replay 的不是同一把
                       （本模块自证「基线起的 cargo 不带门禁加的 release 增量覆盖：…」「基线环境的取法：…」两格必须判错）
  test-closure-ignored  行键不带点名测试的闭包键（自证「改了点名测试调到、不是被改坏的函数：点名它的那一行现判」
                       「改了被改坏函数用的常量（V5 那一类）：原文在那个函数里的两行现判」「改了点名测试够得着、没执行到的函数：那一行现判」三格必须判错）
  baseline-by-tree-snapshot  基线 ok 只按整树快照复用（自证「加一个没人调的函数、同时改一行的替换文：只那一行现判、它的基线复用」
                       「改了点名测试够得着、没执行到的函数：那一行现判，基线按覆盖复用」两格必须判错）
  coveragenotrecorded  抓到之后不记覆盖（自证「头一趟每条点名测试记一份覆盖、同一条测试的几行共用一份」
                       「改了点名测试够得着、没执行到的函数：那一行现判，基线按覆盖复用」两格必须判错；另有记覆盖次数的四格跟着判错：
                       「头一趟起 1 批基线 + 4 条变异，另记 4 份覆盖」「下一趟只跑剩下的 2 条」「插桩编译那一步写出的剖析文件」）
  llvmversionunchecked 不核 llvm 工具与 rustc 的 LLVM 版本（自证「llvm 工具的 LLVM 版本与 rustc 对不上：判红、一条都不跑」那一格必须判错）
  buildprofilesunset   插桩编译那一步不设 LLVM_PROFILE_FILE：构建脚本与过程宏的剖析文件落进它们起的目录
                       （本模块自证「插桩编译那一步写出的剖析文件只落在这一格的临时目录」那一格必须判错）
  keepmutationtmp      每条变异的 TMPDIR 不删（crates-mutation-replay 两份样本都必须判错）
  notrunasmissed       点名的测试打了 NOT_RUN_NO_GPU_ADAPTER 也当没红记（crates-mutation-replay 的 not-run 样本、本模块自证「点名的测试打了 NOT_RUN」「只有本次未跑、一条都没抓到：退 77」「本次未跑的记号不在行首也认」三格都必须判错）
  device-lost-ignored  不认 GPU_DEVICE_LOST 行：点名的测试 FAILED 且自己那一段里、或所有段之外带那一行的也记抓到
                       （crates-mutation-replay 的 device-lost 样本、本模块自证「设备丢失」那一格都必须判错）
  link-failure-as-invalid  替换之后失败在链接（输出里有 error: linking with 与 rust-lld、没有 error[E）也照无效记
                       （本模块自证「链接失败」那一格必须判错）
  link-failure-not-red 报告里链接失败不算红：有链接失败的一趟照样退 0
                       （本模块自证「链接失败」那一格必须判错）
  emptyreplacementrefused  替换文空着也判不成形（crates-mutation-replay 绿样本里的删行变异、code-source-discipline 绿样本里的删行变异、本模块自证「删行变异照常判」那一格都必须判错）
  cargoconfignotcopied 拷每片源码副本时跳过拷贝范围里的 .cargo/（登记漏了它时的样子）：副本里的 cargo 读不到 .cargo/config.toml 的 [env]
                       （本模块自证「拷贝范围含 .cargo/：点名的测试要编 gpu 特性的那一行基线绿、变异抓到」那一格必须判错）
  path-dependencies-not-copied  拷贝范围不调 lib_repository_copy.py 的 copy_closure（不跟 path 依赖、不认编译期与运行期读的仓根文件），只拷登记的那几处
                       （副本里缺 prune-server/，cargo 解析工作区就失败）
                       （本模块自证「拷贝范围跟 path 依赖：…」「拷贝范围跟 path 依赖指到树之外：…」两格必须判错）
  include-literals-from-crate-root  static-check 判拷贝范围 ② 时 include_str!(…)、include_bytes!(…) 的实参也照所在 crate 的根解析，不照源文件所在目录
                       （本模块自证「include 宏的实参照源文件所在目录解析：…」那一格必须判错）
  manifest-path-ignored  不认变异行 cargo 参数里的 --manifest-path：拷贝范围不补那个包目录，单元键照 <根> 那一份工作区算，写坏了也不拒
                       （本模块自证「--manifest-path 静态核：…」「--manifest-path 基线与判红：…」「--manifest-path 单元键照那个包的源码算：…」
                       「--manifest-path 不在仓内：…」四格必须判错）
  product-gpu-feature-unquiesced  产品 GPU 包的 tests-on-a-real-gpu 不当上卡的特性认，只认 gpu：这一包的真显卡对拍行不补 --test-threads、不核用没用上卡
                       （本模块自证「产品 GPU 包（--features tests-on-a-real-gpu）的行：真跑测试的那几次各收尾带 --test-threads 1…」
                       「产品 GPU 包（--features tests-on-a-real-gpu）的行：点名的测试没卡打 NOT_RUN_NO_GPU_ADAPTER…」两格必须判错）
  test-runs-configuration-unpassed  真跑测试的格不往环境里设 SINGLEFS_MULTI_HOST_CONFIG、也不判多机配置在不在
                       （本模块自证「真跑测试的每一次在副本里起的 cargo 环境里有 SINGLEFS_MULTI_HOST_CONFIG…」
                       「多机配置不在：run_stage 退 2…」两格必须判错）
  gpu-unused-as-missed  带 gpu 特性的格判出没红时不核用没用上卡：给定状态全交 CPU 判的也记没红
                       （本模块自证「带 gpu 特性的行：给定状态全交 CPU 判…」那一格必须判错）
  given-states-count-ignored  有 CRASH_POINT_REPLAY_GIVEN_STATES 行就当没用上卡，不读 states_judged_by_the_gpu_here 的数
                       （本模块自证「带 gpu 特性的行：给定状态有 GPU 判过的…照常判」那一格必须判错）
  undefinedidentifiersunchecked  static-check 不判替换文里新引的标识符
                       （本模块自证「static-check 的 UNDEFINED：替换文引了已删的标识符 halved_before 的那一行报一行…」那一格必须判错）
  overlapunchecked     并发领片号那一格不核两次调用重叠没有、照判对错
                       （本模块自证「先起的那次在后起的那次开始之前就退了（样本）：并发领片号那一格认出两次调用没重叠、记本次未跑…」那一格必须判错）
  static-check-registry-from-script-repository  static-check 的拷贝范围与重测试目标读本模块所在那一份仓的登记表，不读被判那棵树的（补丁新登记的行在被判的树里看不见）
                       （本模块自证「static-check 读被判那棵树的登记表：…」两格与「static-check 的 RELEASE：命中而第五段没写 --release 的两行…」那一格必须判错）
  staticusagebare      static-check 参数不对时只打那一行用法，不写缺了什么、不给「→ 怎么办」（本模块自证「static-check 缺 <根>」那一格必须判错）
  tablemissingok       static-check --table 指的文件不在时照旧当空表判、报「CHECKED 0 …」退 0（本模块自证「static-check --table 指的文件不在」那一格必须判错）
  shardrootkeptonsignal  run_pool 不接 TERM / HUP：收到就停，$TMPDIR 下的 singlefs-mutation-replay-* 留着
                       （本模块自证「被 TERM 停：$TMPDIR 下的 singlefs-mutation-replay-* … 停完删掉，退 143」那一格必须判错）
  broken-pool-looked-up-late  「有工作进程中途死了」那一句判 except 的时候才按 concurrent.futures.process 属性找 BrokenProcessPool：进程池还没起就被停时
                       那个子模块没载入，判 except 抛 AttributeError、盖掉 StopRequested（本模块自证「进程池还没起就被 TERM 停」那一格必须判错）
  worker-failure-unconverted  工作进程里一格活抛的异常不换成 WorkerJobFailed（只带原类名与回溯文本两个字符串），原样往外抛、由进程池按它的类 pickle：类定义在按路径载入、
                       只在工作进程里载入过的模块里（临时 prune-server 的 HostAddressUnavailable）时父进程解不开、进程池整个坏掉，报成「有工作进程中途死了」、原因丢了
                       （本模块自证「调用方的 SINGLEFS_MULTI_HOST_CONFIG 进了工作进程的环境：…打原类名（HostAddressUnavailable）与整份回溯…」那一格必须判错，别的格不红）
  dead-worker-cause-unprinted  「有工作进程中途死了」后面不接进程池异常的 __cause__ 文本
                       （本模块自证「进程池坏了且有 __cause__：…」那一格必须判错，别的格不红）
  pool-worker-caller-configuration-dropped  自证里进程池的工作进程（与自证起的子进程）的进程环境不带调用方的 SINGLEFS_MULTI_HOST_CONFIG（Selftest.pool_process_variables 交空）：
                       工作进程里起临时 prune-server 认这一台的地址时认不到调用方指的多机配置，退回本模块所在仓根那一份（源码副本里没有它）
                       （本模块自证「调用方的 SINGLEFS_MULTI_HOST_CONFIG 进了工作进程的环境：…」那一格必须判错，别的格不红）
  term-sent-unwaited   自证「被 TERM 停」那一格不等「临时目录在、假 cargo 在工作进程里挂住」就发 TERM（碰上忙机器：run_pool 还没装好信号处理，默认处理退 -15）
                       （本模块自证「被 TERM 停：$TMPDIR 下的 singlefs-mutation-replay-* 跑的时候在、停完删掉，退 143 …」那一格必须判错）
  shardrootnamedlate   run_pool 建 shard_root 用 tempfile.mkdtemp（目录在 mkdir 里建出来、名字要等它返回才到调用方手里：TERM 落在这两步之间，名字丢了、目录漏在 $TMPDIR）
                       （本模块自证「TERM 落在建 shard_root 的 mkdir 之后、名字交到调用方手里之前：收尾照样删掉」那一格必须判错）
  listitemspipeunhandled  --list-items 不照 SIGPIPE 惯例退（接 | head 时打 BrokenPipeError 的回溯、退 1）
                       （本模块自证「--list-items 接 | head」那一格必须判错）
  extratargetskept     开跑时不回收片号大于这一趟工作进程数的 -w<片号> 编译目录：分片各一份、只增不减
                       （本模块自证「片号大于这一趟工作进程数的 -w<片号> 连同 -coverage 开跑时回收、不大于的留着、打一行删了几份多大」那一格必须判错）
  inusetargetremoved   回收之前不核有没有进程在用（工作目录、命令行、CARGO_TARGET_DIR、在用标记），多出来的照删
                       （本模块自证「多出来的 -w<片号> 有进程在用的不删」那一格必须判错）
  targetdefaulttmp     没设 GATE_MUTATION_TARGET_DIR 时编译目录放 ${TMPDIR:-/tmp}/singlefs-crates-mutation-target（跨轮共用、跑完不删）
                       （本模块自证「跑完运行目录里不剩它建的编译目录（没设 GATE_MUTATION_TARGET_DIR：…）」与「跑完运行目录里不剩它建的编译目录（被 TERM 停）」两格必须判错）
  runtargetkept        跑完不删这一趟自己建的编译目录（连同它所在的 $TMPDIR 下的 singlefs-mutation-replay-*；编译目录跨轮留着）
                       （本模块自证「跑完运行目录里不剩它建的编译目录（没设 GATE_MUTATION_TARGET_DIR：…）」与「跑完运行目录里不剩它建的编译目录（被 TERM 停）」两格必须判错；
                       连带「被 TERM 停（双机分片的驱动停这一份的样子）：…」那一格，$TMPDIR 下的 singlefs-mutation-replay-* 留着）
  runtargetopenfilesunchecked  删这一趟的编译目录之前不核有没有进程开着里面的文件
                       （本模块自证「这一趟的编译目录里还有进程开着文件：不删、报出那个进程」那一格必须判错）
  local-shard-numbering  工作进程的片号不领锁、从本调用的共享计数器现领本地序号：两次并发调用的第 2 片都指向同一个 -w2
                       （本模块自证「同一个共用编译目录上并发的两次调用各起 2 个工作进程：领到的片号集合不相交、四把片号锁都被占」那一格必须判错）
  anchor-stops-table   锚点失效的一行让整张表退出、一个工作进程都不起
  anchor-mid-run-asserts  派活之后工作副本里原文不再恰好一次时 judge_row 抛 AssertionError、整个进程池跟着炸
                       （本模块自证「派活之后锚点失效：那一行记锚点失效，别的行照判照入库」那一格必须判错）
                       （本模块自证「锚点失效按行判：那一行记锚点失效、别的行照跑照入库」那一格必须判错）
  anchor-rows-planned  plan、row-plan、--list-items 不先剔掉锚点失效的行，照常算键：文件不是 .rs 的那一行算得出键、会被分给第二台
                       （本模块自证「row-plan、plan、--list-items 把锚点失效的行单列」那一格必须判错）
  impact-row-dropped   牵连面少算一行（算进的头一行不数）
                       （本模块自证「牵连面：样本 crate 上牵连最大的单元与行数」那一格必须判错）
  reuse-lookup-skipped 复用判据一律不复用（键相同、判过抓到照样现判）
                       （本模块自证 reuse-three-questions 那一格必须判错；reuse-sentinel 那一格要的就是这一趟复用哨兵判红「复用失灵」）
  共用库 scripts/gate/lib_reuse_checks.py 的开关 REUSE_CHECKS_BREAK=rejudged-sentinel-passes（哨兵被现判不红）：本模块自证 reuse-sentinel 那一格必须判错
  uncounted-forced-observed  强制现判那一条落在不算数的结局（VERDICTS_UNCOUNTED：本次未跑）上也当现判结论交哨兵，拿它比库里的「抓到」判成「判定不稳」
                       （本模块自证「复用哨兵：强制现判那一条本次未跑（notrun）：不当现判结论交哨兵，打一行不判、不判成不稳」那一格必须判错）
  exactduplicated      记覆盖那一趟把行里已有的 --exact 与自己补的叠成两个（libtest 报 Option 'exact' given more than once、覆盖记不成）
                       （本模块自证「行里已带 --exact 的变异…」那一格必须判错）
  item-plan-reusable-zero  --list-items --with-keys 的 ITEM_PLAN 里 reusable 一律 0（能复用的行不数）
                       （本模块自证「--list-items --with-keys：每项一行 ITEM_PLAN…」那一格必须判错）
  item-plan-selection-ignored  --list-items --with-keys 带 --rows 也列整表、算整表的键
                       （本模块自证「--list-items --with-keys --rows：只打点名那一行的 ITEM_PLAN…」那一格必须判错）
  item-done-reused-silent  只复用的项不打 ITEM_DONE
                       （本模块自证「真跑：全复用的第二趟每项一行 ITEM_DONE…」那一格必须判错）
  item-done-seconds-whole-run  ITEM_DONE 的 seconds 记自开跑起的整趟，不记这一项自己的
                       （本模块自证「各项的 seconds 是这一项各行自己的秒数相加…」那一格必须判错；「真跑：每项各记各的挂钟…」那一格只判不随机器忙闲变的两条（每项不小于 1 秒、各项之和不超过整趟挂钟），整趟前面的准备耗时长时这个开关不一定判得出，由前一格保证）
  split-result-line-unseen  结果词被测试中途打的行隔开的结果行不接回一行：认不出、基线记成「一行都没出现」
                       （本模块自证「结果词被插进来的 GPU_LIFECYCLE 行隔开时照认基线绿」那一格必须判错）
  closure-error-masked  scripts/mutation/lib_mutation_units.py 里的开关（同样从 GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK 认）：按路径载入的 lib_function_units.py
                       不登记进 sys.modules、算闭包的工作进程把异常原样抛（--list-items 末尾是 PicklingError，真错被盖住）
                       （本模块自证「闭包算不成（并行算…）」与「--list-items 遇到闭包算不成」两格必须判错）
  wgsl-whole-file      scripts/mutation/lib_mutation_units.py 里的开关（同样从 GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK 认）：.wgsl 当数据单元、
                       取整份字节的 sha256（本模块自证「WGSL 按 fn 取键」那一格必须判错）
  forked-parts-by-pool  并行的几处（forked_parts：static-check 的切片、截过的闭包）换回进程池，task 按「模块名.函数名」pickle
                       （本模块自证「按文件路径装成别的模块名（不登记进 sys.modules）装本模块：子进程退 0、本模块没登记进 sys.modules」那一格必须判错：子进程抛 PicklingError）
  static-check-slices-misordered  static-check 逐份、逐行切片并行（static_check_slices），并回时片的次序倒过来
                       （本模块自证「static-check 切片并行与单线程逐行相同（…）」那一格必须判错）
  reachability-rows-unsliced  static-check 可达性逐行那一段（reachability_resolved_rows）不切片，逐行在本进程里算
                       （本模块自证「static-check 可达性逐行那一段（被改坏的单元与点名测试）按行切片并行：…」那一格必须判错）
  workspace-files-unsliced  static-check 建工作区（prebuilt_workspace）不按文件切片，由单元键头一次取时在本进程里逐份切
                       （本模块自证「static-check 建工作区按文件切片并行：…」那一格必须判错）
  rust-file-replaced-on-entry-module  prebuilt_workspace 把子进程切好的 RustFile 的替身换在 lib_function_units.py（再导出）上，不换在 RustWorkspace 所在的模块上，RustWorkspace 读不到，一份都不取
                       （本模块自证「static-check 建工作区按文件切片并行：…」那一格必须判错：并行那一趟取了 0 份）
  mutated-source-reread  scripts/mutation/lib_mutation_units.py 里的开关（同样从 GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK 认）：UnitKeys.source_bytes 不记，
                       被改坏的文件每次现读（本模块自证「static-check 可达性那一段每份被改坏的文件只读一遍…」那一格必须判错）
  specification-lines-product-view  scripts/mutation/lib_mutation_units.py 里的开关（同样从 GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK 认）：规格行（原文落在 verus! 里产品行为视图剥掉的字上的变异行）也取产品行为视图（本模块 row_view 同样认它）
                       （本模块自证「verus! 里的规格行：原文落在产品行为视图剥掉的字上」那一格（selftest_specification_line_cell 调 lib_mutation_units.py 的 specification_line_cell）必须判错）
  rocksdb-entry-unchecked  起任何 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry
                       （本模块自证「RocksDB 预编库的取值缺了：起 cargo 之前判红、退 1、假 cargo 一次都不起」那一格必须判错）。
                       entry 的跳过办法：被判的树里没有依赖 rocksdb 的清单（自证的小仓就是这样），或 PATH 上的 cargo 是带
                       「# rocksdb-prebuilt-entry: selftest-fake-cargo」那一行的假 cargo（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）
复用三问与复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「一」「二」）：
  自证 reuse-three-questions 一格：一份小仓（SELFTEST_UNIT_ROWS 五行）头一趟、同一份输入第二趟、同一个文件里加一个没人调的函数（闭包外）、
    改被改坏的函数 check_root 的别一行（闭包内：原文在它里面的 root-457、root-double 两行现判）各跑一趟，打规范输出
    「REUSE_SELFTEST script=scripts/mutation/crates-mutation-rows.py …」。已有同义的格在这一格里点名、不另留：「输入不变的第二趟：5 行都复用」是 ①，
    「同一个文件里加一个没人调的函数（N45）」与弄坏开关 baseline-by-tree-snapshot 守的「加一个没人调的函数、同时改一行的替换文」是 ②，
    「被改坏函数的别一行变了（S34）」是 ③。
  每一趟 run_stage 开跑前从库里抽哨兵：候选是此刻行键在 records/ 里有「抓到」记录的行（直接读记录，与 row_decision 分开）；抽中强制现判的那一行
    栏写「复用哨兵强制现判」、照现判跑。哨兵被现判、强制现判那一行不是抓到，整道判红；强制现判那一行落在不算数的结局（本次未跑）上的
    不当现判结论交哨兵，哨兵打一行「没报结局，不判」（observe_fresh_outcomes）。种子与条数取环境里的 SINGLEFS_REUSE_SENTINEL_SEED、
    SINGLEFS_REUSE_SENTINEL_COUNT。自证别的格设 SINGLEFS_REUSE_SENTINEL_COUNT=1000（键相同的全当哨兵、不抽强制现判）。
按项机读行（统一任务服务不读它，只有这份脚本的 --selftest 认；一项 = 变异名，同名的几行合成一项，--item 点名同名就是全判）：
  · --list-items --with-keys：每项一行（stdout，次序是变异表的次序，与逐行列出的项一一对应；条数之和等于末行合计的数）
      ITEM_PLAN item=<项名> to_judge=<要现判的条数：键对不上与拒算> reusable=<能复用的条数：键对得上>
    锚点失效的行两样都不算（不跑）。item= 到 to_judge= 之间原样是项名（名字里可以有空格，按行尾三段 to_judge= reusable= 从后往前切）。不起 cargo、不起服务。
  · 真跑（run_stage，ItemProgress）：每项的行全有了结局就打一行（stderr，进度类：带挂钟，不进 stdout，stdout 与进程数无关那一条不动）
      ITEM_DONE item=<项名> seconds=<这一项自己的挂钟秒数> judged=<现判条数> reused=<复用条数> exit=<这一项的退出码>
    seconds 是这一项各行在工作进程里判一行从头到尾的秒数相加（几项并行时各记各的，不记整趟；基线的批几行共用，不记进去；复用的 0）；
    exit：有一行没红、无效、基线不绿、锚点失效的 1，没有而有本次未跑的 77，其余 0（复用的行是抓到）；
    开跑时复用的与锚点失效的项立刻打，现判的项在它最后一行判完时打，被停（只供测试的开关、编译目录停派）没判到的项在收尾补一行（judged=0 reused=0 exit=1）：
    跑到的每一项恰好一行。弄坏开关 item-plan-reusable-zero（ITEM_PLAN 的 reusable 一律 0）、item-done-reused-silent（只复用的项不打 ITEM_DONE）、
    item-done-seconds-whole-run（seconds 记自开跑起的整趟）、item-plan-selection-ignored（--rows 点名也算整表）各让自证 selftest_item_lines_cells 的一格判错。
编译目录（default_shared_target_directory，阶段脚本不另算；自己用完自己删，不跨轮复用）：先认 GATE_MUTATION_TARGET_DIR，设了用它（调用方给的，跑完不删）；
  没设而环境里给了 CARGO_TARGET_DIR（门禁 crates-mutation-replay 文件头声明了会编，统一任务服务给这一次尝试一份，撞上限重来时接着给同一个）用 <它>/singlefs-crates-mutation-target，
  照调用方给的那样办（跑完不删，服务在任务有了结局时删；重来的那一次照用上一次编出的外部依赖与各片产物）；弄坏开关 service-build-directory-ignored 下不认它，
  自证 selftest_service_build_directory_cell。两样都没设：
  没设时 run_pool 在这一趟自己的运行目录（$TMPDIR 下的 singlefs-mutation-replay-*，下称 shard_root）里建 <shard_root>/build/singlefs-crates-mutation-target，
  这一趟的全部编译目录（第 1 片用它，其余片加 -w<片号>，记覆盖的再加 -coverage，先编一份依赖的 -seed、-seed-coverage、-seed-source，片号锁 .slot）都在 <shard_root>/build 里；
  正常结束、判红、被 TERM / HUP 停都在 run_pool 的收尾删掉（scripts/hosts/run_build_directory.py：删之前核本机没有进程开着里面的文件，有就是这一趟的子进程没收干净，
  不删、连同 shard_root 留着、逐个报进程，整道判红）。GATE_CROSS_RUN_TMPDIR 与跨轮缓存家目录不再读。
  回收（reclaim_extra_target_directories）只对调用方给的 GATE_MUTATION_TARGET_DIR 起作用：派活之前，同一层里片号大于这一趟工作进程数的 -w<片号> 与 -w<片号>-coverage 删掉，
  stderr 打一行删了几份、共多大；删之前核没人在用——有进程的工作目录在它底下、命令行里有它底下的路径、环境里 CARGO_TARGET_DIR 指到它底下，或那一片的在用标记
  （工作进程活着时在自己的编译目录里拿着 .singlefs-mutation-worker.lock 的共享锁，回收一侧试拿排他锁拿不到）被别的进程拿着，就留着、逐份写明谁在用。
  片号（claim_worker_slot）经锁领：编译目录那一层里每个片号一把锁文件（第 1 片 <编译目录>.slot，其余片 <编译目录>-w<片号>.slot），
  工作进程启动时从 1 往上试拿排他 flock（不阻塞），拿到第一个空闲的就是它的片号，锁拿到进程退出（stderr 每个工作进程报一行「领到片号 <n>」）。
  调用方给的同一个 GATE_MUTATION_TARGET_DIR 上并发的两次调用因此拿到不相交的片号集合，不再两边的第 2 片都指向同一个 -w2 而被 cargo 的目录锁互相排队。
  片号锁文件建不了、或前 4096 片都被占：报一行，退回本调用内的本地序号。回收同样核片号锁：别的调用的工作进程刚领到的那一片，被占着的不删。
工作进程数（worker_count_for）：四项取最小、至少 1：每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads：SINGLEFS_THREAD_CAP 设了照它，
  否则 .claude/heavy-thread-default；GATE_MUTATION_WORKERS 给了顶掉它）、这一趟要现跑的条数、磁盘余量（再下一段）、内存上限 ÷ 每个工作进程的峰值（下一段「内存上限」）。
  这一趟要现跑的条数就是这一趟要判的单位数（records/2026-10-31-task-server巡检.md）：
  点到的行里没有作数的抓到记录的那几行（run_stage 的 candidates，拒算的照算），不照拿到的上限开满；基线的批（baseline_batches：要现跑基线的几行合成一批，
  批里的行都在这几行里）与覆盖（一行抓到之后补记的那一份、开跑时就要记的那几份，都是这几行的）跟着这几行走，进同一个进程池、不另算工作进程，
  所以工作进程数不超过这几行的行数（统一任务服务给这一道的上限照它自己库里按命令键的记录定，不读这里的算法，也不读 .claude/memory-bounded-parallelism.tsv：那张表只登记哪些入口准许按内存定并发）。
  弄坏开关 workers-beyond-rows 下不收这一项（工作进程数只照线程数、磁盘与内存那几项开，行少而上限大时开得比行多）。
  次序：线程数与条数的小者当天花板交给磁盘那一项，这三项的小者再当天花板交给内存那一项（内存那一项要按真开的进程数算编译并行度，天花板要含磁盘那一项）。取不到每台默认重活线程数（登记文件不在、写坏、
  SINGLEFS_THREAD_CAP 不是正整数）而又没给 GATE_MUTATION_WORKERS 的，整道判红退 2、一条都不跑，不退回按核数算（弄坏开关 workers-by-processor-half：取核数一半、至多 16）。
  不在这里设内存上限：一整条门禁的内存上限仍由统一任务服务按记录给（上面「上限、限时与卡住」）；开几个工作进程照下一段按这一台能给的内存收；服务给的上限装不下时照下一段「撞内存上限」往下减个数，减到 1 个也装不下照开 1 个起跑（真撞了由服务判）；跑着按实测加开见下一段「跑着加开」。
编译并行度（cargo_jobs_for）：每个 cargo 的 CARGO_BUILD_JOBS 取 GATE_MUTATION_CARGO_JOBS（正整数，写法不对整道拒）给了照它，不然 max(1, 每台默认重活线程数 ÷ 工作进程数)
  （compile_thread_budget：default_heavy_threads 取不到整道判红退 2；弄坏开关 workers-by-processor-half：取核数）；
  内存那一项算每个进程数要的编译并行度、run_stage 设 CARGO_BUILD_JOBS 用的是同一个函数。选的是「编译并行度跟着进程数走、算到自洽」，不是固定编译并行度：
  进程数被内存卡小时，固定的编译并行度会让核空着；跟着走则每试一个进程数 n 就照 n 算出编译并行度、再取那个编译并行度下的每进程峰值，满足 n × 峰值 ≤ 上限的最大 n 才开。
内存上限（memory_bound_for）：开几个工作进程按内存 ÷ 每个工作进程的峰值算，与另外三项取最小、至少 1（worker_count_for 里 1 个都放不下按 1 个算、头一行写「那一项不足 1」）。
  run_stage 除的是这一台能给的内存（host_memory_capacity：多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项，scripts/hosts/task_memory.py 的 host_slice_cap_bytes），
  不是服务给的上限：进程数 = min(每台默认重活线程数, 这一台能给的内存 ÷ 每个工作进程开满时的实测峰值)，按核开满、不超这一台的内存。
  这一台那一项取不到时除的是下面取法给的这一条拿到的上限，头一行写取不到的原因。这一条拿到的上限（下面的取法）另当硬顶：开的工作进程 × 峰值装不下时，
  往下减到装得下的个数照跑，减到 1 个也装不下照开 1 个起跑（下面「撞内存上限」）；探针（下面「探针」）按这一条拿到的上限收。
  内存上限的取法（scripts/hosts/task_memory.py 的 task_memory_cap_bytes，本文件同名函数转调它，次序）：① 统一任务服务交的环境变量 TASK_SERVER_MEMORY_CAP_BYTES（这一次尝试的上限，字节十进制整数；写法不对的头一行点名、改走②）；
  ② 仓根多机配置 multi-host.env 的 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项（变异的内存上限直接按仓根目录设置的 50G 走；值是 <地址>=<systemd 写法> 的若干项，
  读法用 scripts/gate/admission.py 的 configuration_value，这一台照多机配置判法 --emit-addresses 认（scripts/hosts/multi_host_addresses.py 的 resolve，整份配置判过才给地址），
  找文件同 test_runs_multi_host_configuration；文件不在、键没写、认不出这一台、读不了都往下走，头一行点名）；
  ③ 自己 scope 的 memory.max（task_memory.py 的 scope_memory_max：/proc/self/cgroup 里 `0::<路径>` 那一行接 /sys/fs/cgroup 再接 /memory.max；只读自己这一层，不往上找父 cgroup）；
  ④ 三样都没有（memory.max 写 max、读不到、写法不认）这一项不收，工作进程数照线程数，头一行写明原因。
  memory.max 是有限值就是硬顶（超了整个 scope 被杀）：①② 取到的值比它大，取 memory.max（两者取小）；①② 都没有，取 memory.max 本身；memory.max 是 max 或读不到，不收窄，照①②。
  头一行写各档读到的值、取了哪一个、为什么（「各档读到：…」）。依据：scope 的真实上限（如 20736M）可以比多机配置写的 50G 小，只按 50G 开会开太多工作进程。
  弄坏开关 memory-max-not-hard-cap：①② 取到了就不看 memory.max。自证用 GATE_MUTATION_CGROUP_LISTING、GATE_MUTATION_CGROUP_ROOT 换掉 /proc/self/cgroup 与 /sys/fs/cgroup（生产不设）。
  每个工作进程的峰值（门禁自己量，不查内存包装的峰值表：各格的 cargo 是工作进程直接起的、不经包装，那张表里永远没有它们）。
  峰值按这一格点名的包（cargo 参数里的 -p）分开记、分开读，这一趟开几个工作进程只看这一趟要现跑的行点名的那几个包（worker_peak_packages）：
  小包的一趟不被大包的历史峰值判死。带不带特性（--features gpu 这一类）不分：同一个包的各种特性共用一份记录，取里面最大的（偏大，不会偏小）。
  没点名包（整个工作区、--manifest-path 指的包）的算 @workspace 这个名字，与点名了的包分开记；-p 写成通配、不是包名的也算 @workspace。
  · 量什么：一个工作进程连同它起的整棵进程树（cargo、rustc、测试二进制、各格的临时 prune-server）此刻的用量，口径是每个进程 /proc/<pid>/status 的
    RssAnon + RssShmem + VmPTE 加总（process_tree_usage；子进程顺 /proc/<pid>/task/<tid>/children 往下找，读不到的刚退出、不算）。
    与 scripts/hosts/run-with-memory-cap.sh 记的「回收不掉的用量」（cgroup 的 memory.current 减 active_file、inactive_file，即 anon + shmem + 内核 + sock）比：
    anon、shmem、页表三样同；差在 ① 内核里不归哪个进程的那几样（slab、内核栈、percpu、sock）这里不算；② 没被任何进程映射着的 tmpfs 文件页这里不算；
    ③ 几个进程共享的匿名页（fork 之后没写过的）这里按进程各算一次，偏大。门禁所在 scope 里另有起这一道的父进程自己的用量，也不在任何一个工作进程的读数里。
  · 怎么量（sample_worker_peak）：每个工作进程起来时（init_worker）在自己里面起一条采样线程，每 WORKER_PEAK_SAMPLE_SECONDS 秒量一次整棵树；
    读数记在此刻在判的那一格点名的包下：进程池里每一格活（baseline_job、mutation_job、coverage_job）判之前用 worker_peak_scope 把这一格 cargo 参数点名的包设进
    WORKER_STATE["peak_scope"]，判完清掉；一格点名几个包的，每个包都记这一格的读数（偏大）；不在判任何一格时（起来时拷副本、先编依赖的拷贝、格与格之间）的读数不记。
    每个包记这个工作进程判这个包的各格里到此刻的最大；这个包头一次量到就写，之后最大涨了 WORKER_PEAK_WRITE_STEP_BYTES 以上就重写
    （同目录临时文件、改名换上），整组被硬杀时已写的留着（记下的比真峰值至多少一个步长；比采样间隔还短的尖峰、比采样间隔还短的一格采不到，推的）。
    一个工作进程判了几个包，就在每个包下各留一份（同一个记录名）。工作进程一趟跑多个包的格时，一格里的读数归这一格点名的包；格与格之间是串行的（一个工作进程一次判一格），不会混。
    量不了（这台内核没有 /proc/<pid>/task/<tid>/children）就不记，stderr 报一行，下一趟照「没有实测」办。
  · 记在哪（record_worker_peak、worker_peaks_directory）：运行记录目录（文件头「判定库」末段，run_records_directory：多机配置 NFS_SHARED_ROOTS 点名的共享根下，
    两台读写同一份，不经 ~ 与 $HOME 取）下 worker-peaks/<包>/<编译并行度>/<起采样的时刻纳秒 20 位>-<进程号>-<8 位随机字符>，内容是一个正整数字节数加换行；
    一个工作进程一趟在每个包下各一份，按包、再按编译并行度分目录。
    测试目标那一级另记一份（worker_peak_target_key、worker_peak_scope）：一格 cargo 参数恰好点名一个包、一个目标时，读数同时记在 <包>--lib、<包>--test--<目标名>、<包>--bin--<目标名>
    （example、bench 同）这个名字下（目录结构、读写、窗口都与包名一样；目标那一级的名字是 worker-peaks/ 下的一级目录，读回来照包名的写法认），包那一份照旧记：
    一个重测试目标（例：验证包不带筛选整个目标跑的那一个）的峰值只定它自己那几行的估值，不再定整个包。不是这一版的布局（编译并行度目录直接在 worker-peaks/ 下的）读成「读不了」，不迁移：删掉 worker-peaks/ 重新量。是运行记录，不进判定库。只有被判的是本仓、--staged 的临时树（GATE_STAGED_FROM），或在临时 prune-server 底下时才写
    （worker_peaks_recorded_for）：门禁样本根不写，样本 crate 的峰值不代表本仓，不在临时服务底下时它的运行记录目录就是本仓那一份。
  · 怎么读（measured_worker_peaks）：每个包、每个编译并行度只看名字排在最后（最新）的 WORKER_PEAKS_RECENT 份，取里面最大的；更旧的读的时候删掉。
    目录还没有是「从没量过」；有一样不是这一版的写法（包目录名不是包名、编译并行度目录名不是正整数、记录名不对、内容不是一个正整数字节数）或读不了，整份记录读不了（所有包一起），
    头一行点名哪一份、为什么，出路是删掉那一份（或整个 worker-peaks/）再跑。
  · 怎么除（worker_peak_for_jobs、memory_bound_for、memory_bound_probing）：天花板 c 是另外三项的小者；n 从 1 试到 c，每个 n 照 cargo_jobs_for 取编译并行度 j，每进程峰值 P(j) 是
    这一趟要现跑的行点名的每个记录名（run_stage 的 candidates 各行 cargo 参数的 worker_peak_keys：有测试目标那一级的取 <包>--…，没有取包名）在 j 下的峰值里最大的；
    一个测试目标那一级的名字自己在 j 或更高的编译并行度没有实测、它的包有记录时，不借包（包的读数是包里最重的一个目标整目标跑出来的，罩住全部目标，借它会让轻目标也只开 1 个），
    取同包别的测试目标那一级在 j 下的读数（照下面 ①②）的中位数（偶数个取中间两个里大的）；同包一份目标那一级的读数都取不到，取起步值
    （worker_peak_starting_bytes：一个 rustc 进程量到的峰值 6.7 GiB；峰值记录不分只编不跑那一类，取不到包自己只编不跑的实测），包在 j 下的读数比起步值小时取包的
    （unmeasured_target_peak）；估小了撞上限由统一任务服务乘 1.5 重排，这里不另兜。包也从没量过的照包算（下面 ③④）。话里点名 --lib 那一级写包名。
    弄坏开关 worker-peaks-unkeyed-by-target：只取包名；unmeasured-target-borrows-package-peak：没有实测的测试目标借它的包（先目标、后包，peak_record_names）。一个记录名在 j 下的峰值取
    ① 它在编译并行度 j 有实测：它最新 WORKER_PEAKS_RECENT 份里的最大；② 没有，而它在更高的编译并行度有实测：借比 j 大的里最小的那一个的（推的：编译并行度越高，每个工作进程的峰值不会越低）；
    ③ 都没有（只有更低编译并行度的实测，或下面 ④ 不收的从没量过），或整份记录读不了：取保守值，就是内存上限本身——按一个工作进程占满整份上限算，只开 1 个；
      要判的几个包里有一个落在 ③，P(j) 就是上限本身；别的包的大小不影响小包的一趟；
      这一口径下只开 1 个，不会因为同时开了几个而撞顶（单开一个就超上限的，开几个都超）。不用写死的默认值。
    ④ 从没量过的包（峰值记录读得了、这个包下一份都没有，new_packages_of）不进 P(j)：P(j) 只取有记录的那几个包；
      要判的包都从没量过，这一项交天花板 c（编译并行度照 c 算）。这几个包由派活时的探针收住（下面「探针」）。
      这一趟不写峰值记录（worker_peaks_recorded_for 是假：门禁样本根）时探针量不到东西，从没量过的包照 ③ 取保守值。
    满足 n × P(j) ≤ 上限的最大 n 就是这一项（一个都不满足交 0）；n 到了天花板 c 还满足，这一项报 ⌊上限 ÷ P(j)⌋（编译并行度照 c 算），不与另外三项并列成「卡在」。
    n 从 1 到 c 每个都试：编译并行度随 n 变，借来的峰值也跟着变，满足与否不随 n 单调。
    GATE_MUTATION_CARGO_JOBS 给了时 j 固定，这一项就是 ⌊上限 ÷ P(j)⌋。
  · 探针（run_pool 派活；new_memory_admission、memory_admits、refresh_memory_admission）：有内存上限这一项、这一趟写峰值记录时，派每一格之前判同时在跑的格数：
    在跑的 k 格连同要派的这一格点名的包合起来，在这一趟的编译并行度 j（CARGO_BUILD_JOBS）下的峰值 P 照 worker_peak_for_jobs 取
    （在 j 与更高的编译并行度都没有实测的包取上限本身），(k + 1) × P ≤ 上限才派；不然在跑的收完再派，排在前头的这一格不让给后面的；一格都没在跑时照派。
    所以没有实测的包的格只在别的格都收完之后单独跑：它跑的那一段整个进程池只有它一格在判，与 ③ 只开 1 个同一口径。
    这一格判的时候采样线程把读数记进 worker-peaks/<包>/<j>/（头一次量到就写）；收完一格、这一趟还有没量到的包时重读峰值记录，量到了就按 (k + 1) × P ≤ 上限放开，
    同时在跑的至多 ⌊上限 ÷ 量到的峰值⌋ 格：放开靠这一趟自己量到的读数，不借别的包的、不猜。比一次采样间隔还短的一格量不到，下一格照样单独跑，直到量到。
    量到的是探针那几格的峰值，同一个包后面更重的一格高出的那一截不在里面（推的），与有实测时只看最新几份记录同一个口径；
    进程池里闲着的工作进程自己的用量（起来时拷副本、格与格之间）不在任何一份读数里，有实测时同样如此。
    要判的包都在 j 下有实测时这一条不收（规划时 N × P ≤ 上限已经成立），也不重读。头一行写哪几个包从没量过、头几格单独跑；放开时 stderr 写一行「包 <名> 量到了（探针）…」。
  · 头一行与收尾一行（stderr）写「内存上限 X（来源）÷ 每个工作进程的峰值 P（编译并行度 j 下：取法）= 同时开 N 个工作进程（n × P = 合计，不超过上限）」，
    取法写取的是哪个包（要判的包不止一个时写「这一趟涉及 N 个包，取其中最大的：包 …」）；卡在这一项时再写多开 1 个要的编译并行度、峰值与合计；
    末了写峰值记录的位置与各包、各编译并行度的读数。stdout 不写（stdout 与进程数无关）。
  依据：查内存包装的峰值表永远查不到（工作进程的 cargo 不经包装），按写死的默认每格峰值（2G）开会低估：只开 4 个（编译并行度约 8）就到 48G，平摊约 12G。
  弄坏开关 memory-cap-ignored：不看内存上限、照线程数开满；worker-peaks-unkeyed-by-package、worker-peaks-unkeyed-by-jobs、worker-peaks-window-unbounded、jobs-feedback-ignored、
  unmeasured-worker-peak-guessed、unreadable-worker-peaks-guessed、worker-tree-self-only、worker-peaks-unrecorded、sample-root-peaks-recorded、
  new-package-run-single-worker、unmeasured-package-not-probed-alone（弄坏开关那一节）。
  自证在 --selftest 的 selftest_memory_cells（上限的取法、撞内存上限）、selftest_worker_peak_cells（量、记、读、除、探针）、selftest_peak_target_cells（按测试目标估、
  没有实测的测试目标取中位数或起步值）与 selftest_worker_growth_cell（跑着加开）。
  撞内存上限（run_stage）：要开的工作进程数 × 那个编译并行度下量到的每个工作进程的峰值比这一条拿到的上限大（按这一台能给的内存开满、服务给的上限装不下，
    或量到的峰值比上限还大、一个都放不下）。先往下减，减到 1 个为止：
    · 编译并行度留在要开的个数那一档（量过的那一档；个数减了照 cargo_jobs_for 会涨，涨上去的那一档常常没量过），每个工作进程的峰值 P 取那一档的；
      找到个数 n 之后编译并行度照 n 个重取（整台线程数 ÷ n，cargo_jobs_for）：重取的那一档（或更高的）有实测、n × 那一档的峰值（加父进程自己）装得下就用重取的，
      没量过或装不下才留在减之前那一档（头一行写「编译并行度照 n 个重取成 …」）。弄坏开关 worker-jobs-held-after-memory-cut：一律留在减之前那一档；
      从要开的个数往下找装得下的最大个数 n（n × P，按这一台能给的内存除时再加起这一道的父进程自己，不超过这一条拿到的上限），
      找到就按 n 个开、照跑，编译并行度照留下的那一档设，头一行改写成「这一轮开 n 个工作进程（因为内存从 <要开的个数> 个减到了 n 个：…，照 n 个开、照跑；减之前…」，
      写上限与两种个数要的内存；收尾一行同样带这一句。
    · 减到 1 个也装不下：照开 1 个起跑、不自己退 250，头一行改写成「这一轮开 1 个工作进程（内存上限装不下一个工作进程的估值：…」（要开的本来就是 1 个）
      或「这一轮开 1 个工作进程（内存从 <要开的个数> 个减到 1 个也装不下：…」，写这一条拿到的上限与估值、「照开 1 个起跑、不自己退 250」，收尾一行同样带这一句；
      编译并行度留在要开的个数那一档。脚本不报自己要多少内存：真撞了上限由统一任务服务判（错误类型表里 memory-cap-hit 那一行，task-server/task_server_lib.py
      文件头「错误类型」），服务乘 1.5 重排，到顶由服务打回 memory-cap-at-maximum；估值偏大（没量过的测试目标取起步值、包里最重的目标）时照开 1 个起跑、跑着按实测加开（下面「跑着加开」）。
    没有实测时取保守值（上限本身），不算进这一条。
    弄坏开关 memory-cap-short-self-exit：一个都装不下时派活之前自己按撞内存上限退 250、不开工（自证 selftest_memory_cells 里「内存上限装不下一个工作进程的估值：…」
    「照这一次服务给的上限开工作进程（1 个也不够）：…」两格必须判错）；
    memory-cap-exits-before-fewer-workers：装不下要开的个数就照一个都装不下办（开 1 个起跑）、不往下找装得下的个数
    （自证 selftest_memory_cells 里「上限只够 1 个工作进程」「上限够 3 个工作进程」两格必须判错）。
  跑着加开（run_pool 的 grown_worker_count、grow_pool；run_stage 交 worker_growth）：探针那一条的前提（有这一条拿到的上限、这一趟写峰值记录）成立、开的个数没到
    线程数、条数、磁盘余量三项的小者（上界）时，工作进程的采样线程判格期间把读数往一个跨进程的最大值里取大（sample_worker_peak 的 seen，这一趟各工作进程到此刻实测峰值的最大）；
    每判完一格、还有不要卡的格排着、没有要卡的格在跑时，取满足「n × 那个最大 + 起这一道的父进程自己 ≤ 这一条拿到的上限」、不超过上界的最大 n，比此刻开的多就定下加到 n 个：
    停派，在跑的收完之后旧进程池收工、换一个开 n 个的新池（fork 之前旧池的管理线程已收掉）；新池的工作进程领片号照旧从 1 往上领，前面那几片的编译目录接着用。
    编译并行度照 n 个重取（cargo_jobs_for），不高过此刻那一档（量到的峰值是此刻那一档下的；编译并行度越低峰值不会越高，推的）；
    之后探针按那个最大收（要判的包都有实测时 (在跑的格数 + 1) × 那个最大 ≤ 上限才派，memory_admits 的 peak_seen）：后面量到更重的格，同时派的跟着少。
    只加一次：那个最大只涨不跌，之后算出来的不会更多。stderr 写「跑着加开：…定下从 m 个工作进程加到 n 个」与换池那一行，收尾一行的「按内存」接上同一句；
    「这一轮 N 个工作进程，每个 cargo 编译并行度 J」写的仍是开跑时的。估值偏大时（没量过的测试目标取起步值、一片里有整个包跑的行）靠这一条把一片开到实测装得下的个数；
    估小了真撞上限由统一任务服务乘 1.5 重排。环境里 GATE_MUTATION_SELFTEST_WORKER_BYTES（只给自证用，生产不设）设了，采样线程量得到时把读数换成它。
    弄坏开关 workers-not-grown-after-measuring：不加开（自证 selftest_worker_growth_cell 里「跑着加开：…」那一格必须判错）。
  底座读数（行名 MUTATION_BASE_READING，stderr 一行）：派活之前量起这一道的父进程（门禁这一层 python）自己此刻的用量，与「先编一份依赖」那一步（工作进程起来之前）
    父进程整棵进程树的峰值，口径同每个工作进程的读数（RssAnon + RssShmem + VmPTE 加总）：
    MUTATION_BASE_READING parent_bytes=<字节或 none> seed_step_tree_peak_bytes=<字节或 none> seed_step=built|not-built
    先编那一步没起（只开 1 个工作进程、够不着外部依赖、弄坏开关 compile-per-worker）时 seed_step=not-built，那一段只量到父进程自己；只报数、不判红。
    这两个数只写进日志给人看，没有程序读；任务服务定上限照它自己库里按命令键的记录，.claude/memory-bounded-parallelism.tsv 不记底座。
算键的内存（KeyContext.prepare、precompute_closures_from_shared_parent、release_key_context）：
  算闭包的子进程是从父进程 fork 的，父进程切好的工作区（UnitKeys 的 RustWorkspace，产品行为视图与形式化视图）每个子进程都带着。子进程读到的页被引用计数抄成私有的，
  循环回收把容器的页逐页抄成私有的，工作区里用到才算的缓存各建一份，整棵树的内存是父进程加上子进程数乘各自抄的量。
  真开子进程时（key_workers 大于 1，这一组至少 PARALLEL_MINIMUM 条）fork 之前依次做三样：
    词法单元共用（share_lexed_tokens，一份工作区一趟）：每个文件的词串进 sys.intern（解释器不写驻留串的引用计数，子进程读它不抄页），行号与括号配对的下标同一个数共用一个对象；取值不变。
    先算一条（precompute_closures 只给请求里排在最前的那一条、就地算）：工作区里用到才算的缓存建在父进程，子进程照抄。这一条算不成（ClosureRefused）的不报，留给进程池那一趟连同点名它的全部行一起报。
    把父进程的对象冻进永久代（gc.collect、gc.freeze，子进程的回收器不走它们），进程池收完 gc.unfreeze。
  子进程算完一格放掉闭包（lib_mutation_units.py 的 _parallel_closure），父进程只留单元号串加共用的单元表（CompactMembers）。这几样都不改闭包的算法，键由闭包自己的成员定。
  派工作进程之前（run_stage，紧挨 run_pool 之前）放掉算键的上下文（release_key_context）：工作进程不用它，要的是 context 里的 row_records（light_components），
    记覆盖的格在自己的副本上现建单元键（record_coverage）。放掉的是工作区、闭包与每一行 Components 的 test_unit（没有读者）；牵连面那一行在放之前算好。
    父进程留下每个文件的单元指纹表（released_unit_keys）：CoveragePlanner 在派活途中照它回答 current_fingerprints，答案与放之前的工作区相同；放的时候工作区里没有的文件交空，
    那一份覆盖照不是此刻的办、多记一份。放完 gc.collect、malloc_trim、gc.freeze（工作进程的回收器同样不走），run_pool 收完 gc.unfreeze。
    stderr 一行机读：MUTATION_KEY_CONTEXT_RELEASE released=yes|no parent_bytes_before=<字节或 none> parent_bytes_after=<字节或 none> workspaces_released=<个> workspaces_left=<个> frozen_objects=<个>；
    workspaces_left 不是 0 是还有东西攥着工作区（工作进程会带着它）；只报数、不判红。工作进程开了才打这一行，没有要现跑的行不打。
  弄坏开关 key-token-values-unshared、key-parent-not-prewarmed、key-pool-not-frozen、key-context-kept-for-workers（弄坏开关那一节末尾）。
磁盘余量（disk_bound_for）：每个工作进程最多占两份编译目录（普通一份、插桩一份，prepare_shard 与 judge_coverage 起的），一份涨到多少按实测峰值算：
  每个工作进程每判完一格（trim_worker_target，清之前）量一次自己那份编译目录的占盘，追加进运行记录目录（文件头「判定库」末段，run_records_directory）下的 target-occupancy.tsv（一行「<时刻秒><TAB>普通|插桩<TAB><字节>」，
  record_target_occupancy），派活之前两种各取最近 20 次里最大的当这一种的涨幅（measured_target_peaks；记录多过 1000 行时只留每种最近 20 行）；
  哪一种还没有读数（头一趟、换了库）的那一种按「编译目录上限」（下下段）算，上限只当清理线。头一行写明每种的涨幅与来源。下面「还能涨的量」「上限」照这个涨幅读；
  开 2 个以上工作进程时先编一份依赖的 -seed、-seed-coverage 两份也每一趟新编，各按一份上限另算（n ≥ 2 才加）。
  派活之前按目标盘（编译目录所在那块）的余量核：片号 1…n 各两份「还能涨的量」（上限减去此刻已占，目录还没建的是整个上限；每一趟自己建的都还没建）加起来，
  n ≥ 2 再加先编那两份，再加备用 GATE_MUTATION_DISK_RESERVE（默认 30G，推的），
  不超过余量的最大 n 就是这一项；n 从 1 数到前两项最小的那个数再多 1（放得下再多 1 个才不与前两项并列成「卡在」）。一个都放不下按 1 个开（头一行写「那一项不足 1」）。
  头一行写：磁盘余量多少、来源、备用多少、每份涨到多少（含先编那两份），n 个共还要多少，再多 1 个要多少、放不下，所以给 n 个；「卡在「磁盘余量」这一项」。
  余量的来源：外层给了 GATE_MUTATION_DISK_FREE（systemd 写法，例 400G）就用它（双机分片的驱动、自证用）；没给读编译目录往上第一层存在的目录所在的盘此刻可用的字节（statvfs）；
  两样都读不到（写法不认的整道拒；statvfs 失败的）没有这一项，头一行写「磁盘余量没读到」及原因，工作进程数不按磁盘收。已有的编译目录按此刻占盘量，
  回收（reclaim_extra_target_directories）发生在这一步之后，所以只回收得掉的多出来的目录不算作余量（偏保守）。
  依据：第二台 9 个工作进程各一份 -w<片号> 加各一份插桩，会把 1.8T 的盘写满（tempfile.mkdtemp 抛 OSError: [Errno 28]）；压到 4 个仍涨到普通 39–71G、插桩 30–68G。
派活那一段（run_pool）：基线的批、变异与覆盖进同一个进程池，
  互相独立的几格按工作进程并行（同时在跑至多 worker_count 格）；一个工作进程只有一份编译目录（-w<片号>），cargo 锁着它，同一个工作进程里的几格只能逐格跑。
编译目录上限（trim_target_directory、trim_worker_target，GATE_MUTATION_TARGET_DIR_CAP，默认 64G 是推的）：每个工作进程每判完一格（基线的批、一条变异、一组覆盖），
  量一次它自己那份编译目录（判基线与变异的普通那份，判覆盖的插桩那份）此刻的占盘；超过上限就清掉 <目录>/<档>/incremental 与 <目录>/<目标三元组>/<档>/incremental
  （rustc 每编一次改坏的副本往里加一个会话目录，只增不减；两格之间没有 cargo 在这份目录上跑），清完回读再量，stderr 报清了几处、共多大、清后多大。
  清完增量仍超上限的，再清 <目录>/<档>/deps 与 <目录>/<目标三元组>/<档>/deps 里同名产物的旧版本（stale_dependency_artifacts）：文件名「<名字>-<16 位单元哈希>[.<后缀>]」
  按（目录、名字、后缀）归堆，每堆按修改时刻只留最新那一版；同一个包换了特性或档编出的另一版也算同名，之后的格要用它时 cargo 照「产物不在」重编（慢，判定不变）。
  清完回读仍超上限就停派：工作进程往这一趟 shard_root 里的停派目录（target_over_cap_directory）写一句，run_pool 每收一格看一眼，有就不再派活、等在跑的收完，
  整道判红，拒绝说明逐份点名清后仍超线的编译目录、出路是调大上限或换盘（判完的格已入库，再跑一趟只跑剩下的）。清不成（权限、文件被占）只报、不丢这一格的判定、不停派。
  本机量到的一份普通编译目录 77G，其中 debug/incremental 13G、debug/deps 30G、release 30G，所以清增量产物不保证清到线下；磁盘余量那一项按实测峰值算，不依赖清得掉。
调试信息与 release 增量（cargo_debug_overrides）：这一道起的每条 cargo（基线、变异的先编与跑、插桩编译与插桩跑、先编一份依赖）都带
  CARGO_PROFILE_DEV_DEBUG、CARGO_PROFILE_TEST_DEBUG、CARGO_PROFILE_RELEASE_DEBUG、CARGO_PROFILE_BENCH_DEBUG = CARGO_DEBUG_SETTING（false）：cargo test 用 test 档（承 dev），
  cargo test --release 用 release 档（bench 承 release），四档都设，顶掉调用方设的同名变量与仓根 Cargo.toml 的 [profile.release] debug；
  另带 CARGO_PROFILE_RELEASE_INCREMENTAL = CARGO_RELEASE_INCREMENTAL_SETTING（true）：release 档开增量编译，改坏一处只重编改到的那几个代码分区。只设在起 cargo 的那一份环境里。
  命中重测试目标、这一趟补了 --release 的行（「重测试目标只许 release」）走的就是 release 档，同样关调试信息、开增量；档本身照 --release 进行键的「开关与特性」。
  基线例外：基线起的 cargo 不带 CARGO_PROFILE_RELEASE_INCREMENTAL 这一项（baseline_cargo_environment：调用方自己设了的值照放回、没设就不放），release 档照仓根 Cargo.toml 不开增量；
  调试信息那四项照带。为什么见下面「基线连调用方的判定库服务」。
  编译档的取值（调试信息、增量编译、opt-level、debug-assertions、overflow-checks、lto、panic、codegen-units，环境变量、cargo 配置与仓根 Cargo.toml 三处）都不进行键：
  KeyContext 不收（BUILD_ENVIRONMENT_BREAKS 的弄坏开关另收，lib_mutation_units.py 的 toolchain_lines 交那几行，它文件头「工具链行」）。
  run_stage、plan、row-plan、--list-items 同一份。
  调试信息只进 DWARF 段，不改判定；增量编译在 codegen-units 没写时把代码分区数从 16 换成 256，所以经 codegen-units 进键；插桩的覆盖映射不靠 DWARF（量过：一个探针包关掉调试信息、带 -Cinstrument-coverage
  编出来的测试二进制没有 .debug_info 段，llvm-cov export 照样报执行过的函数计数 1、没执行的 0）。
  量到的（本机，singlefs-core 的测试目标冷编）：开着时写盘 916 MB、编译目录 826 MB、测试二进制 79 MB；关掉后 510 MB、421 MB、17 MB。
先编一份依赖（prepare_seed、sync_dependency_artifacts）：每个工作进程各有一份源码副本与一份编译目录（-w<片号>，插桩的再加 -coverage），冷的那几份各自把外部依赖
  （Cargo.lock 里带 source 的包：libc、vstd、wgpu、rocksdb 这一类）编一遍。这一趟开 2 个以上工作进程、要现跑的行（基线与变异）选的包顺 Cargo.lock 够得着外部依赖时，
  派活之前先在 <编译目录>-seed（插桩的 <编译目录>-seed-coverage）里按组各编一次（组 = 一行 `--` 之前的参数只留选包、特性、档与目标三元组，seed_group_of；
  先编那一组用它头一行自己选的目标，cargo 才连开发依赖一起编），每个工作进程起来时把这两份里外部依赖的产物拷进自己的两份编译目录（大小与修改时刻都对得上的不拷，拷的保留修改时刻）。
  插桩那一份只编这一趟可能要记覆盖的行（现判的行里覆盖记录不是此刻的那几行，CoveragePlanner.predicted_rows）所在的组。
  工作区里的包（Cargo.lock 里的 path 包）照旧各编各的、不拷：编出来的二进制把 CARGO_MANIFEST_DIR 编死在里面（env!），换了路径的副本拿着拷来的产物 cargo 照样判 Fresh
  （cargo 1.98 量过），跑的时候读的就是先编那一份副本的文件；unit-graph 这类在跑的时候读别的包的源码，改坏的包不是它的依赖时它不重编，判定就变了。
  先编那一份的源码副本 <编译目录>-seed-source 每一趟新建（调用方给了 GATE_MUTATION_TARGET_DIR 时跟着它留着）、按内容同步（.claude/rules/test-script-discipline.md「编译与算法都要快」那一节的修改时刻规矩）：内容没变的文件不重写、修改时刻不动；
  变了、新加、删掉的文件所在的包（crates/<目录>，有 Cargo.toml 的）整包把修改时刻摸成此刻，不在哪个包底下的改动摸全部包。先编每一组都带 --message-format=json-render-diagnostics，
  摸过的包编出来的每一个产物，在普通（或插桩）那一份里头一次出现时都要不是 fresh（那个包有 Compiling 行）；有一个头一次出现就是 fresh，整道判红、一条都不跑（stale_seed_artifacts）。
  同一份里前面的组已经以非 fresh 编出过的同一个产物，后面的组复用它、cargo 报 fresh 是对的，不算坏：产物认 cargo JSON 里的单元身份（seed_unit_identity：package_id、manifest_path、
  目标的 kind、crate_types、name，档、特性、filenames、executable；filenames 里 deps/ 下的文件名带 cargo 按包、特性、档、目标三元组、rustc 参数与依赖算的单元哈希，同一份编译目录里两个单元不会同名），
  只按包名不放行：同包另一套特性或档编出来的单元是另一个身份，头一次出现就是 fresh 照判坏。普通与插桩两份各记各的。
  建的时候（同步源码、编）拿 <编译目录>-seed.lock 的排他锁，工作进程拷的时候拿共享锁（调用方给的同一个 GATE_MUTATION_TARGET_DIR 上几次调用共用这一份时也不互相顶掉；产物按单元哈希起名）。
  并行度 = 每台默认重活线程数（default_heavy_threads；这时还没有工作进程在跑，整台的线程都给它），GATE_MUTATION_CARGO_JOBS 设了用它（seed_jobs）。
  拷（copy_seed_dependencies、sync_dependency_artifacts）：每个工作进程起来时各拷各的（几个工作进程同时拷），一个工作进程里普通、插桩两份要拷的文件
    再分到 CARGO_BUILD_JOBS（每台默认重活线程数 ÷ 工作进程数）条线程上拷。
  先编没编成（退出码不是 0）的那一份只报、不判红，工作进程照旧自己编依赖（判定不受影响，只是慢）；拷的时候出错同样只报。编完照「编译目录上限」清一次那两份的增量产物；
  「磁盘余量」给这两份各留一份上限（每一趟新编）。只开 1 个工作进程、要现跑的行都够不着外部依赖（树里没有 Cargo.lock 的自证小仓就是这样）时不起，stderr 写明没起的原因。
  量到的（本机，singlefs-core 的测试目标、关掉调试信息）：冷编一份写盘 510 MB、CPU 31.6 秒；拷依赖产物 175 个文件 98 MB；拷完之后在另一份新副本里编，
  只有 singlefs-core、singlefs-format、singlefs-replay-prefix 三个工作区包重编，依赖 16 个全 Fresh，写盘 343 MB、CPU 21.3 秒（与拿着整份旧编译目录的工作进程 343 MB、20.6 秒相同）。
测试线程数：不压。起 cargo test 时 RUST_TEST_THREADS 与仓里各测试读的线程数变量（SINGLEFS_THREAD_CAP 等）调用方设了什么就原样透传；只有 CARGO_BUILD_JOBS（编译并行度）按每台默认重活线程数 ÷ 工作进程数收。
变异复跑的崩溃判定：起 cargo test 真跑测试的每一次（变异、记覆盖的插桩跑；只编不跑的 --no-run 与基线的批不起）各起一份自己的临时 prune-server
  （mutation_run_prune_server：照 scripts/prune-server/prune_server_client_selftest.py 的 TestServer 起，socket、状态目录与库都在它自己的临时目录里），
  那一次环境里连哪一处（SINGLEFS_PRUNE_SERVER_SOCKET、SINGLEFS_PRUNE_SERVER_TCP_ADDRESS）与临时服务的标记（PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET）都指它，
  跑完停服务、删目录：走判定库的 验证包用例在变异版上判出的崩溃结论只落进那一份临时库，不读真库、不写真库（变异复跑本就该现判）。
  服务二进制取 PRUNE_SERVER_BINARY（绝对路径、可执行；与客户端自证的 resolve_binary 认同一个变量），没设由 run_pool 开派之前照本模块所在仓的 prune-server/
  编一份进这一趟的 <shard_root>/prune-server-build（cargo build --release --offline，收尾随 shard_root 删）；取不到整道判红、一条都不跑。
  起服务经 <shard_root> 里写的一份包装：先设父进程死亡信号再换成真二进制，工作进程被停或被硬杀时（finally 不跑）服务跟着停，不留孤儿。
  弄坏开关 mutation-run-on-real-service：不起临时服务，照调用方的环境连那一份服务，变异版判出的崩溃结论写进真库同一把键下。
基线连调用方的判定库服务（run_baseline_batch 给 run_cargo_command 的 on_real_service、mutation_run_prune_server、baseline_cargo_environment）：基线的批在没改坏的副本上跑，不起临时 prune-server，
  起 cargo 时环境里连哪一处照调用方的（门禁在统一任务服务底下跑时是全仓那一份真服务；自证里是自证所在的那一份临时服务）：走判定库的 验证包用例判过的崩溃状态块按块键从那一份读，
  没判过的判了写进去，与门禁 crash-point-replay 读写的是同一份、同一种键；基线跑的是没改坏的源码，它判出的结论就是该进那一份库的。
  崩溃判定的版本含工具链与 cargo 解析后的 profile：增量编译开着时 codegen-units 的生效值是 256、关着是 16，两把版本不同、读不到对方的块；
  所以基线起的 cargo 不带门禁加的 CARGO_PROFILE_RELEASE_INCREMENTAL（上面「调试信息与 release 增量」），--release 的基线与门禁 crash-point-replay 的 cargo test --release 同一把版本。
  变异与记覆盖的插桩跑照旧各起自己的临时服务：变异版判出的崩溃结论不进真库；改坏的若是崩溃枚举的流水线文件（录写表、排计划、领块、存库、对账那几份，不进判器的键），键不变、读真库会读到没改坏那一版的结论。
  变异版那一趟一律现判：按键只读复用真库里基线那一份结论，要服务或客户端有只读的读法，它们没有。
  scripts/mutation/prove-red.sh 的证红同样分：基线连调用方的服务、不带 release 增量覆盖（同用 baseline_cargo_environment）；改坏之后起的每一次 cargo 各起自己的临时服务
  （同用 mutation_run_prune_server 与 mutation_run_prune_server_binary，那份脚本文件头「判定库与复用」），判出的崩溃结论不进调用方的库。
  弄坏开关 baseline-on-temporary-service：基线也起自己的临时服务，判过的块读不到、判完的写不进调用方那一份；
  baseline-release-incremental-kept：基线起的 cargo 也带 CARGO_PROFILE_RELEASE_INCREMENTAL，版本与 crash-point-replay 的不同、读不到它判过的块。
libtest 的结果行（`test <全名>[ - should panic] ... <结局>`）与汇总行只在 scripts/gate/lib_libtest_result_lines.py 解析一份（results_by_test 调它）；
  结果词被测试中途打的行隔开的写法（`test <名> ... ` 之后隔着别的输出、结局另起一行；测试里的线程直接写标准输出时出现）由 joined_result_lines 先接回一行再交它：
  结局单独一行，起头与结局之间的行不算，起头之后先来了别的结果起头的不接，单独的结局行多出来没处接的整段不接（同 lib_libtest_result_lines.py 的 account_binary）。
  弄坏开关 split-result-line-unseen 见上面的开关表。
  它的弄坏开关 LIBTEST_RESULT_LINES_BREAK 写在它文件头；本模块自证里认得它的那一格：
  LIBTEST_RESULT_LINES_BREAK=should-panic-in-name  本模块自证「should panic 的用例红了判抓到、点名省掉模块路径只对上那一个」那一格必须判错
下面五个开关（磁盘余量、编译目录上限）一行一个，下一行起写打开后必须判错的自证项：
  disknotchecked       工作进程数不看磁盘余量：第二台 9 个工作进程把盘写满
    本模块自证「磁盘余量放不下 16 份…」「已有的编译目录只算还能涨的…」「磁盘余量交给 run_stage…」「外层没给磁盘余量时读编译目录所在的盘…」四格必须判错
  targetcapignored     编译目录超过上限也不清增量产物：产物只增不减
    本模块自证「编译目录超过上限：清掉两处增量产物…」「清完仍超线：只清增量产物…」「编译目录上限交给 run_stage…」「编译目录清完增量仍超上限：deps 里同名产物只留最新一版…」
    「编译目录清完仍超上限（1K）：头一格判完就停派…」五格必须判错
  depsnotpruned        清完增量仍超上限也不清 deps 里同名产物的旧版本
    本模块自证「编译目录清完增量仍超上限：deps 里同名产物只留最新一版…」那一格必须判错
  overcapdispatched    清完仍超上限只报、照派
    本模块自证「编译目录清完仍超上限（1K）：头一格判完就停派…」那一格必须判错
  diskbudgetbycap      磁盘预算不看实测峰值、每份编译目录一律按上限算
    本模块自证「磁盘预算按实测峰值算：…」那一格必须判错
下面十四个开关（调试信息与 release 增量、行键里不进的几样、先编一份依赖）一行一个，下一行起写打开后必须判错的自证项：
  debuginfo-kept       起 cargo 时不设 CARGO_PROFILE_<档>_DEBUG：调试信息开着
    本模块自证「门禁起的每条 cargo 都关了调试信息」那一格必须判错
  release-incremental-unset 起 cargo 时不设 CARGO_PROFILE_RELEASE_INCREMENTAL：release 档照 Cargo.toml 不开增量
    本模块自证「门禁起的每条 cargo 都开了 release 档的增量编译」那一格必须判错
  build-environment-keyed  KeyContext 另收 lib_mutation_units.py 的 toolchain_lines 交的行（编译器与链接器相关的环境变量、cargo 配置的 build / target / env、根清单的 lints）：
                       build_environment_lines_for_break 交这些行，行键、基线键、覆盖号各收一份（KeyContext.components）
    本模块自证「改编译器的环境变量 RUSTFLAGS 之后行键不变」「改编译器与链接器相关的环境变量之后样本每一行的行键、基线键、覆盖号都不变」
    「改 cargo 配置的 build、target、env 之后样本每一行的行键、基线键、覆盖号都不变」「改根清单的 lints 之后样本每一行的行键、基线键、覆盖号都不变」四格必须判错
  profile-keyed        lib_mutation_units.py 的 toolchain_lines 另收环境里的 CARGO_PROFILE_*、cargo 配置与根 Cargo.toml 的 [profile] 的原文行，KeyContext 把 toolchain_lines 交的行收进行键
    本模块自证「改编译档的 debug 之后行键不变」「改编译档的 incremental 之后行键不变」「改编译档的 opt-level、debug-assertions、codegen-units 之后行键不变」
    「改配置文件与清单里的编译档、线程数之后行键不变」四格必须判错
  build-jobs-keyed     lib_mutation_units.py 的 toolchain_lines 另收环境里的 CARGO_BUILD_JOBS 与 cargo 配置 [build] 的 jobs，KeyContext 把 toolchain_lines 交的行收进行键
    本模块自证「改线程数之后行键不变」「改配置文件与清单里的编译档、线程数之后行键不变」两格必须判错
  toolchain-version-keyed  lib_mutation_units.py 的 toolchain_lines 另收 rustc -vV、cargo -V 原样各行，KeyContext 把 toolchain_lines 交的行收进行键
    本模块自证「改 rustc、cargo 版本之后行键不变」「假 cargo 的 -V 换一行之后样本每一行的行键、基线键、覆盖号都不变」「没有 rustc 与 cargo 也算得出行键」三格必须判错；
    toolchain_lines 的行里本来就有环境变量、cargo 配置与 lints，「改编译器的环境变量 RUSTFLAGS 之后行键不变」与上面 build-environment-keyed 的另外三格跟着判错
  dependency-versions-keyed  lib_mutation_units.py 的开关（它文件头「依赖版本行」）：Cargo.lock 里点名外部包的版本行并进点名测试的闭包键
    本模块自证「改 Cargo.lock 里的依赖版本之后行键不变」那一格必须判错
  driver-and-library-keyed  仓根下驱动（scripts/mutation/ 的 crates_mutation_rows_*.py、crates-mutation-rows.py、lib_mutation_units.py）与算键库（scripts/gate/lib_function_units*.py）
    各份文件的内容并进行键、基线键、覆盖号（KeyContext.components）
    本模块自证「改驱动与算键库的函数体：…每一行的行键、基线键、覆盖号都不变」那一格必须判错
  compile-per-worker   不起先编那一份，每个工作进程自己编外部依赖
    本模块自证「4 个工作进程：外部依赖只在先编那一份里各编一次」那一格必须判错；「先编那一份的源码按内容同步」「先编那一份的修改时刻被留住」两格跟着判错（先编那一趟没起）
  workspaceunitsshared 从先编那一份拷产物时连工作区的包一起拷
    本模块自证「先编那一份只拷外部依赖的产物」那一格必须判错
  seedmtimekept        同步先编那一份的源码时，内容变了的文件写进去之后修改时刻改回原来的、一个包都不摸（修改时刻规矩坏了的样子）
    本模块自证「先编那一份的源码按内容同步：改过的包重编、整道照常」那一格必须判错（先编那一趟判红）
  seedfreshunchecked   先编那一趟不核摸过的包有没有重编
    本模块自证「先编那一份的修改时刻被留住：先编那一趟判红、一条都不跑」那一格必须判错
  seedfreshacrossgroups 先编那一趟每一组各判各的：前面的组已经编过的产物，后面的组里 fresh 也记成坏的
    本模块自证「先编那一份有两组：后一组复用前一组编出来的产物、cargo 报 fresh 不算坏，整道照常」那一格必须判错（先编那一趟判红）
  seed-copy-serial     工作进程从先编那一份拷依赖产物时在一条线程上逐个拷
    本模块自证「工作进程拷依赖产物分到几条线程上同时拷」那一格必须判错
  seed-copy-threads-unregistered 没给 CARGO_BUILD_JOBS 时工作进程拷依赖产物固定用 1 条线程，不取每台默认重活线程数 ÷ 工作进程数、取不到也不拒
    本模块自证「工作进程拷依赖产物的线程数：没给 CARGO_BUILD_JOBS 取每台默认重活线程数 ÷ 工作进程数…」那一格必须判错
下面这个开关（基线的批按工作进程并行），下一行起写打开后必须判错的自证项：
  baseline-one-at-a-time 基线的批一次只派一批（在跑的那一批收完才派下一批）
    本模块自证「4 个工作进程、8 批基线：同时在跑的基线不少于工作进程数…」那一格必须判错
边跑边收、边跑边报那几个开关（文件头同名一段），一行一个，下一行起写打开后必须判错的自证项：
  lifecycle-lines-kept  GPU 阶段现场行不滤，整份收进结果
    本模块自证「边跑边收：假命令往 stderr 写 200 MiB…」「被停（工作进程的默认处理）…」「被停（父进程接住的信号）…」「边跑边报：子进程一直有新输出时…」四格必须判错
  lifecycle-filter-any-stream  stdout 里同样前缀的行也滤
    本模块自证「边跑边收：stdout 里同样前缀的行不滤…」那一格必须判错
  lifecycle-prefix-loose  行里任何位置有这几个字就滤
    本模块自证「边跑边收：行中间夹着 GPU 阶段现场字样的不滤…」「边跑边收对拍：同一批行滤与不滤…」两格必须判错
  lifecycle-prefix-renamed  滤的前缀与 crates 那侧打的不是同一个，一行都滤不到
    本模块自证「边跑边收：假命令往 stderr 写 200 MiB…」「边跑边收：滤的前缀与 crate singlefs-verification…」「被停（工作进程的默认处理）…」「被停（父进程接住的信号）…」「边跑边报：子进程一直有新输出时…」五格必须判错
  lifecycle-tail-dropped-on-stop  被停时不打留下的行
    本模块自证「被停（工作进程的默认处理）…」「被停（父进程接住的信号）…」两格必须判错
  progress-line-only-on-output  这一段子进程没有新输出就不打进度
    本模块自证「边跑边报：子进程 10 个间隔不出输出时照样每个间隔打一行进度，不丢…」那一格必须判错
  progress-line-unwritten  一行进度都不打
    本模块自证「边跑边报：子进程一直有新输出时…」「被停（工作进程的默认处理）…」「被停（父进程接住的信号）…」三格必须判错（被停那两格靠进度行确认读线程收完了才发信号）
  heartbeat-prefix-dropped  进度行不带 TASK_SERVER_HEARTBEAT 前缀
    本模块自证「边跑边报：子进程一直有新输出时…」「边跑边报：子进程 10 个间隔不出输出时…」两格必须判错
  computing-line-unwritten  在算行一行都不打
    本模块自证「在算行：子进程 6.5 秒里只出被滤掉的 GPU 阶段现场行…」「在算行：TMPDIR 是基线批的前缀…」「在算行：TMPDIR 是覆盖组的前缀…」三格必须判错
  computing-line-without-output  子进程这一段一个字都没出也打在算行
    本模块自证「在算行：子进程 5 秒里一个字都不出…」那一格必须判错
  computing-cell-baseline-unrecognized  在算行的格名不认基线批的前缀 baseline-
    本模块自证「在算行：TMPDIR 是基线批的前缀…」「在算行：judge_row、run_baseline_batch、judge_coverage 建临时目录的前缀…」两格必须判错
  computing-cell-coverage-unrecognized  在算行的格名不认覆盖组的前缀 coverage-
    本模块自证「在算行：TMPDIR 是覆盖组的前缀…」「在算行：judge_row、run_baseline_batch、judge_coverage 建临时目录的前缀…」两格必须判错
  computing-cell-prefix-renamed  在算行的格名把变异行的前缀认成 row-（judge_row 建目录的前缀是 line-）
    本模块自证「在算行：子进程 6.5 秒里只出被滤掉的 GPU 阶段现场行…」「在算行：judge_row、run_baseline_batch、judge_coverage 建临时目录的前缀…」两格必须判错
重基线批切块、没有用时的与长测试单列那五个开关（文件头「基线」），一行一个，下一行起写打开后必须判错的自证项：
  heavy-batch-unsplit  重的批不切，也不分长短与有没有用时
    本模块自证「重基线批切块：同键 40 行…」「重基线批切块：同一个过滤词的行…」「重基线批切前切后对拍…」「重基线批被整组硬杀…」四格必须判错，
    「没有用时的与长测试各占一块：…」「没有用时的与长测试单列一块的跑法：…」两组与「基线一块判完记下点名测试的用时：…」一格同样判错
  heavy-batch-row-dropped  切块时只留头一块、后面的行丢了
    本模块自证「重基线批切块：同键 40 行…」「重基线批切块：同一个过滤词的行…」「重基线批切前切后对拍…」「重基线批被整组硬杀…」四格必须判错，
    「没有用时的与长测试各占一块：…」「没有用时的与长测试单列一块的跑法：…」两组同样判错
  long-baseline-not-isolated  没有用时的与长的都不单列，照旧只按过滤词个数切
    本模块自证「没有用时的与长测试各占一块：…」「没有用时的与长测试单列一块的跑法：…」两组与「基线一块判完记下点名测试的用时：…」一格必须判错（别的格都绿）
  unknown-timing-merged-as-short  没有用时的测试照短的并进按个数切的块（长的照旧单列）
    本模块自证「没有用时的与长测试各占一块：…」「没有用时的与长测试单列一块的跑法：…」两组与「基线一块判完记下点名测试的用时：…」一格必须判错（别的格都绿）
  baseline-timing-unwritten  基线一块判完不记用时
    本模块自证「基线一块判完记下点名测试的用时：…」一格必须判错（别的格都绿）
  上面 baseline-batch-composition、records-written-at-end 两条另各多判错一格：「重基线批切前切后对拍…」「重基线批被整组硬杀…」
算键的内存那四个开关（文件头同名一段），一行一个，下一行起写打开后必须判错的自证项：
  key-token-values-unshared  fork 之前不把词串与行号换成共用的对象
    本模块自证「算键的进程池：真跑的小仓上词串进了 sys.intern…」「算键的进程池：真跑的小仓上先共用了词法单元…」两格必须判错
  key-parent-not-prewarmed  fork 之前父进程不先算一条
    本模块自证「算键的进程池：假驱动上父进程先算一条…」「算键的进程池：真跑的小仓上先算了一条…」两格必须判错
  key-pool-not-frozen  fork 之前（算键的进程池与工作进程的进程池）不把父进程的对象冻进永久代
    本模块自证「算键的进程池：假驱动上 fork 之前冻…」「算键的进程池：真跑的小仓上冻了对象…」「派工作进程之前：run_stage 放掉了工作区…」三格必须判错
  key-context-kept-for-workers  派工作进程之前不放算键的上下文
    本模块自证「派工作进程之前：假驱动上放掉工作区…」「派工作进程之前：run_stage 放掉了工作区…」两格必须判错
重测试目标只许 release 那两个开关（文件头同名一段；写成 CRATES_MUTATION_ROWS_BREAK=<项>，GATE_MUTATION_BREAK 同样认），一行一个，下一行起写打开后必须判错的自证项：
  CRATES_MUTATION_ROWS_BREAK=release-unforced  命中重测试目标而 cargo 参数没带 release 的行不补 --release：崩溃枚举用例目标与 gpu 行在 debug 档跑
    本模块自证「重测试目标只许 release：命中的两行（crash-case 目标、gpu 特性）起的每一次 cargo 都带 --release、只带一个，不命中的那一行一次都不带，stderr 点名补了哪两行」那一格必须判错
  CRATES_MUTATION_ROWS_BREAK=release-unchecked  static-check 不判命中重测试目标而第五段没写 --release 的行
    本模块自证「static-check 的 RELEASE：命中而第五段没写 --release 的两行（gpu 特性、crash-case 目标）各报一行、带依据、跟一行写出路，退 1；命中写了的与不命中的不报，两行都写上之后退 0」那一格必须判错
  CRATES_MUTATION_ROWS_BREAK=no-default-features-unadded  验证包只点名集成测试目标的行不补 --no-default-features：这一趟起的 cargo 照编 26 个装置二进制
    本模块自证「验证包的测试目标不编装置二进制（判法）：…」「验证包的测试目标不编装置二进制（一趟真跑）：…」两格必须判错
上卡的格与服务给的编译目录那几个开关（文件头「上卡的格一格接一格」「编译目录」；写成 CRATES_MUTATION_ROWS_BREAK=<项>，GATE_MUTATION_BREAK 同样认），一行一个，下一行起写打开后必须判错的自证项：
  CRATES_MUTATION_ROWS_BREAK=gpu-cells-dispatched-in-parallel  上卡的格不挪到最后：与不上卡的同一队、照空位并行派
    本模块自证「上卡的格一格接一格：…」那一格必须判错
  CRATES_MUTATION_ROWS_BREAK=plain-cells-cards-visible  不上卡的格起 cargo 时不藏卡：环境里没有那几个藏卡的变量
    本模块自证「不上卡的格看不到卡：…」那一格必须判错
  CRATES_MUTATION_ROWS_BREAK=service-build-directory-ignored  没设 GATE_MUTATION_TARGET_DIR 时环境里给的 CARGO_TARGET_DIR 也不认：同两样都没设时一样在这一趟的 shard_root 里建、跑完删
    本模块自证「服务给的编译目录：…」那一格必须判错
只供测试的开关：GATE_MUTATION_STOP_AFTER_ROWS=<k> 现跑的变异判完 k 条就不再派活、等在跑的收完，整道判红退出（造「跑到一半被杀」）。
"""
import ast  # noqa: F401
import collections  # noqa: F401
import concurrent.futures  # noqa: F401
import contextlib  # noqa: F401
import fnmatch  # noqa: F401
import gzip  # noqa: F401
import hashlib  # noqa: F401
import heapq  # noqa: F401
import importlib.util  # noqa: F401
import io  # noqa: F401
import json  # noqa: F401
import multiprocessing  # noqa: F401
import os
import random  # noqa: F401
import re  # noqa: F401
import shutil  # noqa: F401
import signal  # noqa: F401
import stat  # noqa: F401
import subprocess  # noqa: F401
import sys
import tempfile  # noqa: F401
import threading  # noqa: F401
import time  # noqa: F401
import tomllib  # noqa: F401
import traceback  # noqa: F401
from concurrent.futures.process import BrokenProcessPool  # noqa: F401

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))   # 拆出的各份与本文件同目录；按文件路径载入本文件的别处也靠这一行 import 得到它们
from crates_mutation_rows_base import (SCRIPT_DIRECTORY, STAGE_REPOSITORY, preflight, HeavyThreadDefaultError, default_heavy_threads,
    multi_host_addresses, LIBTEST_RESULT_LINES_PATH, _LIBTEST_RESULT_LINES_SPECIFICATION, libtest_result_lines, MUTATION_UNITS_PATH,
    _MUTATION_UNITS_SPECIFICATION, mutation_units, write_atomically, REUSE_CHECKS_PATH, _REUSE_CHECKS_SPECIFICATION, reuse_checks, SCRIPT_KEY, STAGE,
    STAGE_SCRIPT, MODULE_PATH, MODULE_SOURCE_PATHS, SELFTEST_FIXTURES_PATH, POOL_ERROR_CHILD_PROGRAM, STAGE_CHILD_PROGRAM, ROCKSDB_PREBUILT_PATH,
    TABLE, LIBRARY_DIRECTORY_NAME, ROW_KEY_FORM, BASELINE_KEY_FORM, COVERAGE_KEY_FORM, LLVM_TOOLS, LLVM_TOOLS_VARIABLE, COVERAGE_TARGET_SUFFIX,
    TARGET_DIRECTORY_NAME, RUN_BUILD_DIRECTORY_NAME, RUN_BUILD_REMOVAL_FAILURES, WORKER_TARGET_LOCK_NAME, WORKER_SLOT_LOCK_SUFFIX,
    WORKER_SLOT_SEARCH_LIMIT, COVERAGE_FLAG, BUILD_PROFILES_DIRECTORY_NAME, CARGO_DEBUG_PROFILES, CARGO_DEBUG_SETTING,
    CARGO_RELEASE_INCREMENTAL_SETTING, SEED_TARGET_SUFFIX, SEED_SOURCE_SUFFIX, SEED_LOCK_SUFFIX, SEED_SELECTOR_SWITCHES,
    SEED_VALUE_SELECTOR_SWITCHES, SEED_DROPPED_SWITCHES, SEED_VALUE_DROPPED_SWITCHES, SEED_VALUE_KEPT_SWITCHES, UNIT_ARTIFACT_NAME,
    UNIT_DIRECTORY_NAME, SEED_COPY_TEMPORARY_SUFFIX, REUSE_COLUMNS, COLUMN_TITLES, REASONS_SHOWN, REUSABLE_VERDICT, RECORD_HEADER,
    IGNORED_WHEN_COPYING_INTO_EACH_SHARD, CARGO_CONFIGURATION_DIRECTORY, DEFAULT_LONG_ROW_SECONDS, STOP_GRACE_SECONDS,
    PER_MUTATION_TEMPORARY_DIRECTORY_NAME, SIZE_TEXT, SIZE_UNIT_BYTES, TARGET_DIRECTORY_CAP_VARIABLE, DEFAULT_TARGET_DIRECTORY_CAP,
    DISK_FREE_VARIABLE, DISK_RESERVE_VARIABLE, DEFAULT_DISK_RESERVE, TASK_MEMORY_CAP_VARIABLE, WORKER_PEAKS_RECENT, WORKER_PEAK_SAMPLE_SECONDS,
    WORKER_PEAK_WRITE_STEP_BYTES, WORKER_PEAK_RECORD_NAME, WORKER_PEAK_WORKSPACE_SCOPE, WORKER_PEAK_PACKAGE_NAME, SLICE_MEMORY_CAP_KEY,
    CGROUP_LISTING, CGROUP_ROOT, TARGET_DIRECTORY_COPIES_PER_WORKER, INCREMENTAL_DIRECTORY_NAME, VERDICTS_COUNTED, VERDICTS_UNCOUNTED,
    LINK_FAILURE_LINE, LINK_FAILURE_LINKER, TEST_NAME_FORM, NOT_RUN_MARK, NOT_RUN_WORD, NOT_RUN_TEST_FIELD, NOT_RUN_REASON_FIELD, DEVICE_LOST_LINE,
    DEVICE_LOST_LINE_OUTSIDE_SECTIONS, LIBTEST_SHOW_OUTPUT, GPU_TEST_FEATURES, GPU_TEST_THREADS_RELEASED_TARGETS, GPU_ROUTE_UNITS, GPU_ROUTE_FEATURE,
    REACHABILITY_GPU_ROUTE_STOPS, REACHABILITY_CPU_FALLBACK_ROOTS, REACHABILITY_CPU_CONFIGURATION_TEXT, REACHABILITY_HOWTO, COLLISION_REFUSAL_PREFIX,
    BYTES_PER_STAT_BLOCK, EXIT_SELECTION_ONLY, STOP_SIGNALS, SHARD_ROOT_PREFIX, PRUNE_SERVER_TEMPORARY_SERVICE_SCRIPT, PRUNE_SERVER_BINARY_VARIABLE,
    PRUNE_SERVER_BUILD_DIRECTORY_NAME, PRUNE_SERVER_WRAPPER_NAME, PRUNE_SERVER_PARENT_DEATH_WRAPPER, LITERAL_OR_COMMENT_START,
    PATH_ATTRIBUTE_BEFORE_LITERAL, PATH_ATTRIBUTE_AFTER_LITERAL, INCLUDE_MACRO_BEFORE_LITERAL, STAGE_VARIABLES, Row, FIELD_NAMES, REPLACEMENT_FIELD,
    ROW_PROBLEM_KINDS, _ADMISSION_MODULE, REPOSITORY_COPY_PATH, COPY_REBUILD_HOWTO, REGISTERED_COPY_RANGE_REASON, _REPOSITORY_COPY_MODULE,
    _PRUNE_SERVER_TEMPORARY_SERVICE_MODULE, RELEASE_ONLY_TESTS_PATH, _RELEASE_ONLY_TESTS_MODULE, RELEASE_REGISTRY_ROOT, RELEASE_HOWTO,
    RELEASE_REGISTRY_HOWTO, STATIC_CHECK_REGISTRY_HOWTO, admission_module, BREAK_VARIABLES, break_is, say, note, cargo_debug_overrides, baseline_cargo_environment,
    build_environment_lines_for_break, unescape, malformed_reason, read_table, copied_paths, under_temporary_prune_server, Components, Decision,
    split_at_double_dash, WORKER_STATE, format_size, task_memory_cap_bytes, process_usage_bytes, output_progress_seconds, bytes_of_size,
    CLOSURE_REFUSED_HOWTO, print_rejection, merge_progress_variable, merge_step_open, merge_step_part, merge_step_row_done, merge_step_close,
    inserted_before_double_dash, _REACHABILITY_PARALLEL, ITEM_PLAN_PREFIX, ITEM_DONE_PREFIX, ITEM_EXIT_BY_VERDICT, FAKE_CARGO, FAKE_LLVM_TOOL,
    PARALLEL_REASON_FORM, ITEM_PLAN_LINE, ITEM_DONE_LINE)  # noqa: F401
from crates_mutation_rows_judging import (anchor_broken_text, executed_set_is_current, baseline_reuse_problem, baseline_arguments,
    row_libtest_selection, libtest_selects, row_baseline_output, joined_result_lines, results_by_test, output_tail, failure_excerpt,
    baseline_problem, with_shown_output, not_run_marks_in, not_run_reason, device_lost_line_of_test, device_lost_line_outside_sections,
    verdict_of_run, caught_text, single_full_name, prepare_shard, gpu_test_run, with_gpu_test_threads, libtest_stdout_sections, named_test_stdout,
    given_states_gpu_counts, gpu_card_unused_reason, run_cargo_command, uncounted_verdict_keeps_counted_record, write_record_quietly, how_text,
    run_baseline_batch, judge_batch, judge_row)  # noqa: F401
from crates_mutation_rows_support import (selftest_write, selftest_script_registration_table, selftest_registration_table_path,
    selftest_copy_registration_table, selftest_write_stage_registration_table, build_selftest_repository, selftest_new_library_copy, selftest_one_forced,
    selftest_forced_row_name, plain_invocations, coverage_invocations, mutation_invocations, selftest_verdict_view, source_hash, replace_in_file,
    replace_in_part, per_row_how, run_command_capturing)  # noqa: F401
from crates_mutation_rows_static import (repository_copy_module, release_only_tests_module, judged_registry_root, release_registered_targets, release_forced_row,
    default_features_off_row, release_unwritten_rows, copied_with_path_dependencies, is_copied_into_each_shard, rows_outside_copy,
    manifest_directory_of, copied_with_manifest_directories, unit_keys_under_manifest, unit_keys_for_directory, stray_escapes, source_text_outcome,
    anchor_problem_of, anchor_problem, anchor_problems_by_line, anchor_broken_rows, row_problems, package_directory_of, rust_identifier_set,
    package_identifier_set, rust_unbound_references, replacement_undefined_identifiers, replacement_candidates, replacement_unbound,
    forked_slice_by_pool, forked_slice_child, forked_parts, static_check_slices, read_replaced_source, identifier_set_task, replacement_unbound_task,
    replacement_undefined_identifiers_of_rows, string_literals, literal_value, escaping_literals_of_file, escaping_literals, read_bytes,
    selftest_static_check_parallel_repository, selftest_static_check_parallel_compare, selftest_include_macro_literal_cell,
    selftest_run_overrides_outside_service_cell)  # noqa: F401
from crates_mutation_rows_units import (memory_bounded_jobs, row_content_hash, row_identity, utc_now_text, refuse_overrides_outside_service,
    records_home, library_directory, open_library, run_records_directory, snapshot_of, snapshot_identity, key_workers, row_view,
    row_unit_keys, built_workspaces_of, share_lexed_tokens, new_key_pool_facts, precompute_closures_from_shared_parent, released_unit_keys,
    release_key_context, KeyContext, index_entry, explain_fresh, row_decision, read_timing, selftest_seed_short_timings, library_of,
    selftest_library, selftest_child_module_loader, selftest_key_pool_child_program, selftest_key_pool_fake_cells, selftest_key_pool_closure_keys,
    selftest_key_release_child_program, selftest_key_release_fake_cell, selftest_write_timings, selftest_reuse_cells,
    selftest_multiple_packages_cell, selftest_specification_line_cell, selftest_mutate_plan_test_service_code_cell, selftest_uncounted_keeps_counted_cells,
    driver_and_library_digests)  # noqa: F401
from crates_mutation_rows_baseline import (batch_key, baseline_batches, baseline_chunk_filter_limit, heavy_baseline_batch, split_baseline_rows,
    baseline_long_test_seconds, baseline_test_groups, baseline_group_basis, split_rows_by_length, baseline_chunk_label, baseline_chunks, crate_of,
    rows_grouped_by_crate, rows_grouped_by_configuration, dispatch_order, selftest_libtest_result_cell, selftest_split_result_line_cell,
    selftest_coverage_exact_cell, selftest_not_run_item_name_cell, selftest_not_run_mid_line_cell, selftest_device_lost_marker,
    selftest_device_lost_output, selftest_device_lost_cell, selftest_heavy_chunk_cells, selftest_heavy_chunk_repository,
    selftest_baseline_timing_write_cell, selftest_heavy_chunk_resume_cell)  # noqa: F401
from crates_mutation_rows_targets import (StopRequested, raise_stop_requested, own_workers_stopped_on_stop, remove_owned_build_root,
    deferred_write_library, crash_domain_narrowing, narrowed_outcome_text, narrowed_sentinel_off_reason, narrowed_sharding_refusal,
    unwritten_library, slot_lock_path, claim_worker_slot, default_shared_target_directory, run_build_target_directory,
    planned_run_build_target_directory, remove_run_build_root, hold_worker_target_lock, path_mentioned_in, processes_using_directories,
    worker_lock_is_held, file_lock_is_held, allocated_bytes_of_tree, reclaim_extra_target_directories, target_reclaim_lines, size_setting_problems,
    target_directory_cap_bytes, worker_target_directories, disk_free_bytes, disk_bound_for, incremental_directories_of, trim_target_directory,
    stale_dependency_artifacts, trim_worker_target, target_over_cap_directory, target_over_cap_reports, target_occupancy_file,
    record_target_occupancy, measured_target_peaks, selftest_target_directory_cells, selftest_run_target_directories, selftest_slot_overlap_problem,
    selftest_concurrent_slot_pair, selftest_worker_slot_cells, environment_records, selftest_write_random, selftest_target_measure_cadence_cell,
    selftest_target_cap_cells, selftest_target_prune_cells, selftest_disk_peak_cells)  # noqa: F401
from crates_mutation_rows_memory import (WORKER_COUNT_VARIABLE, CARGO_JOBS_VARIABLE, THREAD_SETTING_VARIABLES, compile_thread_budget, cargo_jobs_for, worker_peaks_directory, worker_peak_packages, worker_peak_target_key,
    worker_peak_keys, peak_key_label, peak_record_names, worker_peak_scope, new_worker_peak_record_name, record_worker_peak,
    worker_peaks_recorded_for, process_kind, process_tree_anon_bytes_by_kind, base_reading_line, process_tree_usage, sample_worker_peak,
    measured_worker_peaks, worker_peak_for_jobs, peak_at_jobs, worker_peak_starting_bytes, unmeasured_target_peak, new_packages_of,
    unmeasured_packages_at, memory_bound_for, host_memory_capacity, memory_bound_probing, cell_peak_packages, new_memory_admission, memory_admits,
    refresh_memory_admission, worker_threads_bound, worker_count_for, grown_worker_count, selftest_worker_count_cells, selftest_memory_cells,
    selftest_worker_peak_cells, selftest_dispatch_order_cell, selftest_peak_target_cells, selftest_record_tiny_peak, selftest_peaks_read,
    selftest_jobs_after_memory_cut_cell, selftest_worker_growth_cell, selftest_disk_cells)  # noqa: F401
from crates_mutation_rows_runs import (measure_and_remove_temporary_directory, gpu_lifecycle_prefix, new_output_collection, collect_output_lines,
    output_progress_line, computing_line_seconds, computing_cell_label, computing_tests_of, output_computing_line, wait_for_output_readers,
    report_lifecycle_tail, print_lifecycle_tail_on_default_stop, collected_stderr_text, run_capturing_output, gpu_released_test_targets,
    card_pool_only_test_targets, test_runs_multi_host_configuration, prune_server_temporary_service_module, mutation_run_prune_server,
    mutation_run_prune_server_binary, write_row_timing, coverage_build_settings, relative_inside, executables_of_build, judge_coverage,
    record_coverage, selftest_gpu_test_threads_pool_targets_cell, selftest_gpu_cell_dispatch_cells, selftest_service_build_directory_cell,
    selftest_capture_child_program, selftest_capture_run, selftest_output_collection_cells, selftest_output_stop_run, selftest_output_stop_cells,
    selftest_output_progress_cells, selftest_temporary_directory_templates)  # noqa: F401
from crates_mutation_rows_pool import (init_worker, seed_group_of, read_lock_file_packages, path_packages_reaching_external, seed_groups, seed_lock,
    files_under, package_directories, sync_seed_source, seed_unit_identity, stale_seed_artifacts, seed_jobs, build_seed_group, prepare_seed,
    dependency_artifact_entries, dependency_info_sources, sync_dependency_artifacts, copy_one_dependency_file, copy_dependency_files,
    seed_copy_threads, dependency_copy_jobs, copy_seed_dependencies, write_baseline_timings, baseline_job, mutation_job, coverage_job,
    new_shard_root_path, pool_cell_runs_on_gpu, card_hiding_environment, pool_cell_job, run_pool, broken_pool_error_type,
    run_pool_reporting_dead_worker, WorkerJobFailed, selftest_stop_before_pool_cell, selftest_worker_failure_report_cell, build_seed_repository, with_break, selftest_seed_cells, selftest_seed_jobs_cell,
    selftest_seed_copy_threads_cell, seed_artifact_message, selftest_seed_group_cells, selftest_pool_parallel_cells,
    selftest_mutation_run_service_cells, selftest_output_verdict_cell, selftest_key_pool_repository_cells, selftest_heavy_chunk_verdict_cell,
    selftest_long_baseline_chunk_cells, selftest_long_baseline_chunk_run_cell)  # noqa: F401
from crates_mutation_rows_plan import (read_selection, prechecks, light_components, baseline_check, plan_rows, refuse_colliding_row_keys,
    gpu_route_enabled, gpu_route_feature_word, gpu_route_package, gpu_feature_on, gpu_route_feature_word_of, with_feature, gpu_route_rows,
    ReachabilityConfigurationError, ReachabilityNotJudged, reachability_units_named, reachability_table_names, reachability_names_missing,
    reachability_stop_pairs, cpu_configuration_writer, cpu_fallback_root_in, reachability_cut_answer, _reachability_cut_task,
    reachability_cut_answers, reachability_precompute, reachability_closure, source_text_reader, reachability_refused_reason,
    reachability_plans_by_closure, workspace_file_task, workspace_file_items, prebuilt_workspace, reachability_row_answer,
    reachability_resolved_rows, reachability_of_rows, decide_rows, impact_unit_name, impact_summary, selftest_gpu_route_feature_cell,
    selftest_gpu_test_threads_cell, selftest_release_repository, selftest_release_forced_cell, selftest_default_features_off_cell,
    selftest_debuginfo_cells, selftest_key_release_cells, selftest_row_key_collision_cell, selftest_reuse_fix_cells,
    selftest_driver_and_library_not_in_key_cell, selftest_test_service_code_key_cell, selftest_build_environment_and_version_cells)  # noqa: F401
from crates_mutation_rows_stage import (item_exit_code, ItemProgress, item_plan_lines, run_stage, run_stage_rows, sentinel_item,
    observe_fresh_outcomes, plan_reuse_sentinel, CoveragePlanner, llvm_tools, report, reuse_summary_text, write_not_run_record,
    unfiltered_rows_judged, unfiltered_rows, selftest_signal_and_pipe_cells, selftest_killed_run_resume_cell, selftest_shard_root_window_cell,
    selftest_cargo_configuration_cell, selftest_product_gpu_feature_cells, selftest_gpu_configuration_cells, selftest_link_failure_cell,
    selftest_sentinel_uncounted_cell, build_coverage_repository, selftest_build_profile_cell)  # noqa: F401
from crates_mutation_rows_commands import (static_check_usage_problem, command_static_check, command_library_directory,
    plan_context, command_row_plan, command_plan, command_list_items, command_item, command_sample_tools, command_llvm_tools,
    selftest_undefined_identifier_cell, selftest_reachability_static_check, selftest_reachability_cells, selftest_reachability_refused_cells,
    WRAPPED_FUNCTION_MODULES, selftest_static_check_parallel_cell, selftest_closure_refused_cell, selftest_release_static_check_cell,
    selftest_release_judged_tree_registry_cell, selftest_unfiltered_static_check_cell, selftest_path_dependency_copy_cells, selftest_manifest_path_cells, selftest_crash_domain_narrowing_cells,
    selftest_unit_key_cells, selftest_parallel_key_cell, selftest_wgsl_unit_key_cell, selftest_item_lines_cells, selftest_anchor_and_impact_cells)  # noqa: F401
from crates_mutation_rows_driver import (Selftest, selftest_loaded_by_path_parallel_cell, selftest_cell_functions, selftest_cell_function_names,
    selftest_cell_functions_without_work, selftest_timing_sensitive_cell_functions, selftest_break_cell_table, selftest_break_switches,
    selftest_selected_cell_functions, selftest_break_cell_table_cell, selftest_process_alive, selftest_probe_sleeper, selftest_probe_sleeper_pid,
    selftest_probe_process_state, selftest_probe_leftover_process, selftest_probe_overtime, selftest_probe_interval,
    selftest_probe_concurrent_interval, selftest_probe_concurrent_interval_companion, selftest_probe_functions, selftest_probe_intervals_overlap,
    selftest_cell_scheduling_cells, selftest_cell_call, selftest_cell_limits, selftest_cell_jobs_for, selftest_cell_jobs,
    selftest_heaviest_cell_functions, selftest_cell_dispatch_order, selftest_group_members, selftest_stop_group, selftest_describe_members,
    selftest_cell_child, selftest_fork_cell_functions, selftest_run_cell_functions, selftest_merge_cell_outcomes, selftest_options,
    selftest_fixture_values, FIXTURE_MODULES, load_selftest_fixtures, run_selftest, run_selftest_cells)  # noqa: F401

def main(arguments):
    if arguments[:1] == ["--selftest"]:
        # 判定库那几格写进 prune-server：只在临时服务上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service），
        # 不是就一格都不跑，不写真服务
        client = mutation_units.Library.client()
        service_problem = client.selftest_service_problem()
        if service_problem:
            print(f"  ✗ crates-mutation-rows.py 自证没跑：{service_problem}")
            print(f"     → 怎么办：{client.SELFTEST_SERVICE_REMEDY}")
            return 2
        return run_selftest(arguments[1:])
    if arguments[:1] == ["--item"]:
        return command_item(arguments)
    commands = {"static-check": command_static_check,
                "library-directory": command_library_directory, "row-plan": command_row_plan, "plan": command_plan,
                "llvm-tools": command_llvm_tools, "sample-tools": command_sample_tools, "--list-items": command_list_items}
    if not arguments or arguments[0] not in commands:
        print("  ✗ 用法：crates-mutation-rows.py {static-check|library-directory|row-plan|plan|llvm-tools|sample-tools|--list-items|--item} … 或 --selftest",
              file=sys.stderr)
        print("     → 怎么办：各子命令的参数见文件头；整张表的复跑只经门禁 crates-mutation-replay.sh（bash .claude/gate.d/crates-mutation-replay.sh）", file=sys.stderr)
        return 2
    return commands[arguments[0]](arguments[1:])


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
