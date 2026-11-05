#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 scripts/gate/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判
# gate-stage: 崩溃点重放与登记的崩溃枚举用例（包里全部测试函数按单元键复用，登记的崩溃枚举用例逐条按崩溃状态块的单元键跑）
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: code-source-discipline.sh 与 model-differential-and-scenarios.sh 也调 scripts/crash/crash-case-check.py，但调的是 file-names、one-scenario 两样，分归代码类与 harness 类；这里开跑前只调 placement、modules 两样，判的是崩溃枚举用例与 验证包测试文件，红了就不起 cargo；crash-case-must-red、crash-knobs、crash-case-provenance 三格另调 must-red、knobs、provenance 三样（崩溃枚举用例的必红变异、崩溃点测试的旋钮表、合表各项在来历表里的行，别的门禁都不调）
# gate-covers: 崩溃点重放
# gate-cell: crash-case-replay 包里全部测试函数按单元键复用（登记的崩溃枚举用例经 --skip 排掉）、登记的崩溃枚举用例逐条按崩溃状态块的单元键跑（块键相同的复用、缺的当场判）
# gate-cell: feature-gated-tests 带上 验证包 tests/ 里 cfg 点到的特性另跑一趟（每条测试函数按单元键复用）：挂特性的测试文件逐个跑到或复用的用例不为 0，GPU 对拍没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑
# gate-cell: product-gpu-real-card-tests 起产品 GPU 包（singlefs-background-gpu）带 tests-on-a-real-gpu 特性才编的真显卡对拍测试（经清场、--test-threads 1，每条测试函数按单元键复用）：对拍判红整格红，没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑
# gate-cell: crash-flow-coverage 崩溃流覆盖表：没有登记表 crash-entries.tsv 外的入口、登记的入口都有覆盖行、StepKind 每个成员都被一条流罩住
# gate-cell: crash-ignored-nodes ignore 开着的节点（ignored_point、registered_segment、not_expanded）不比基准 crash-ignored-nodes.tsv 多
# gate-cell: invariant-zero-evaluations 评估 0 次的（流，不变量）对都在带理由的基准 invariant-zero-evaluations.tsv 里
# gate-cell: crash-case-must-red 每条崩溃枚举用例在 crates/mutations.tsv 里至少有一行变异点名它的测试目标
# gate-cell: crash-knobs 旋钮登记表 crash-knobs.tsv 每行取值与依据不空、出处路径在
# gate-cell: crash-case-provenance 崩溃点重放合表里调 enumerate_crash_states 的崩溃枚举项与放量分项，在来历表 crash_point_replay_provenance.rs 里都有一行
#
# 只有一条路：不带 --check 与 --item 就是九格全跑，门禁照单元键自己选要判的（键相同的复用，键对不上的现判），没有轻重、快慢之分，也没有「全部现判」的跑法。
# 指定范围只有两种：--check 点名格，--item 点名项。项有两类（下面「逐项点名」）：崩溃枚举用例（只限定按块那一路跑哪几条，按条复用那一趟照跑）、
#   挂特性的测试文件（feature-gated:<测试文件名>，只限定 feature-gated-tests 格编哪几个、跑哪几个）。
#
# 逐项点名（.claude/rules/test-script-discipline.md「入口脚本：不带参数就是不指定范围」，统一任务服务按项估时、分片读下面两种机读行）：
#   项的写法：
#     crash-case:<用例名>               登记的崩溃枚举用例（.claude/gate.d/stage-inputs.tsv 键是 crash-case: 的行；--item 里 crash-case: 可写可不写），归 crash-case-replay 格的按块那一路；
#     feature-gated:<测试文件名>        验证包 tests/ 顶层挂特性 cfg 的一个测试文件（list_feature_gated_test_files 现读，例 feature-gated:gpu_pair.rs），归 feature-gated-tests 格；
#     feature-gated:lib                 那一格带特性那一趟里的 --lib 一段（库在这一组特性下的单测）。
#     feature-gated 的项合起来就是整格：不点就是全部；点了的只编、只跑点名的测试目标（--lib 只在点了 feature-gated:lib 时带），特性照整格那一组
#     （tests/ 里全部挂特性的文件现读的并集），所以每条测试函数的单元键与整格里的相同，点名单跑一项与它在整格里跑的判定相同、判定库里互相复用。
#     不带特性那一趟打了 NOT_RUN_ 的补判只在同一趟跑了 crash-case-replay 格时才有，点了 feature-gated 的项那一趟不另加目标。
#   没给 --check 时：点了 feature-gated 的项而没点崩溃枚举用例的，只跑 feature-gated-tests 一格；点了崩溃枚举用例的照旧九格（feature-gated 的项照样只限定那一格）。
#   给了 --check 而点了 feature-gated 的项、点的格里没有 feature-gated-tests：退 2（点的项不在点的格里）。项写错（没登记的用例、tests/ 里没有挂特性的这个文件）退 2，列出可点的项。
#   样本根的 .gate-items（一行一项，# 之后与空白不算）当作命令行上的 --item：只在被判的根是样本（不是本阶段所在的仓，也没设 GATE_STAGED_FROM）、命令行没给 --item 时认；
#     stage-selftest.sh 起样本只给根，样本要点名就靠它（同门禁 code-tooling 的 .gate-items）。
#   机读行（写法同 scripts/crash/crash-point-replay.sh 文件头「输出」，task-server/task_server_lib.py 的 ITEM_PLAN_LINE、ITEM_DONE_LINE 认）：
#     ITEM_PLAN item=<项> to_judge=<件数> reusable=<件数>   --list-items --with-keys，stdout，列项之后每项一行；同时给了 --item 的只打点名的项。
#       feature-gated 的项：件是那个测试目标里编得进的测试函数（admission.py test-plan 带这一组特性、在清掉 SINGLEFS_CRASH_AMPLIFICATION_* 与
#       SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 的环境里算，登记的崩溃枚举用例与放量分项经 --exclude-registered 不算），键对不上与拒算的算要现判、键相同的算复用；
#       崩溃枚举用例：块键只有引擎按整库数、没有按项的读法，一条用例一件、算要现判（同崩溃点重放入口）。test-plan 算不成的那几项不打 ITEM_PLAN、另打一行「算不出」。
#     ITEM_DONE item=<项> seconds=<秒> judged=<现判的件数> reused=<复用的件数> exit=<0|1|77>   真跑，stdout，跑完每项一行：
#       feature-gated 的项在那一格收尾时打：秒数是这一项那几段 libtest 汇总行「finished in <秒>s」之和（只算测试跑的时间，编译几项共用一份、不分摊；一条都不用跑的项 0），
#       件数照这一格的计划；exit 1 是那一格判红（这一项里有 FAILED 的测试函数，或认不出红在哪一项时点到的每一项都记 1），77 是这一项跑到加复用 0 条或有没卡没跑的 GPU 对拍，别的 0。
#       崩溃枚举用例在按块那一路每条跑完时打：秒数是这一条从开跑到跑完的挂钟，全部块都复用的记 judged=0 reused=1、别的 judged=1 reused=0，判红 exit 1。
#
# 出口 4 那五格（.claude/kb/milestone/03-third-txn.md「出口」的出口 4、六·1、六·3、六·4；用户 2026-10-20 定的 K1–K7 原话在
#   records/2026-10-18-里程碑三收尾与近期欠账.md「用户 2026-10-19 弹窗定出口 4 七条」那一条）：判法不在这里，覆盖、ignore、0 次三格在
#   scripts/crash/crash-flow-coverage.py（文件头「判法」），必红世界、旋钮两格在 scripts/crash/crash-case-check.py 的 must-red、knobs；这里只备行、调判法、记结局。
#   覆盖、ignore、0 次三格读这一趟的覆盖行（CRASH_FLOW_COVERAGE …）与计数行（I-x.y=评估/违例/不适用）：
#     按条复用两趟（crash-case-replay 格不带特性那一趟、feature-gated-tests 格带特性那一趟）的日志按测试二进制交 crash-flow-coverage.py collect
#     （复用的测试这一趟不跑、不打行：整份复用的测试二进制取回判定库里同一组键整份现判时存下的行，一半复用、库里没有的记「行不全」），
#     每条判绿的崩溃枚举用例的日志交 crash-flow-coverage.py extract（来源取它的测试目标）；没判绿的用例记「行不全」。
#     crash-case-replay 没点、或判红时三格本次未跑；没点 feature-gated-tests、或它判红时挂特性的那几份记「行不全」。
#     collect 退 75（存取覆盖行时 prune-server 不可用）时三格照不可用办（「退出码」75：只有不可用退 75，另有真红退 1），别的非 0 是判法脚本出错、三格判红。
#     这一趟的记录放这一趟的日志目录 <common-dir>/singlefs-crash-point-replay-logs/<开跑时刻>-<pid>/flow-records.tsv（跑完不删），打一行 CRASH_FLOW_RECORDS_FILE <路径>
#     （崩溃点重放入口照这一行把它并进覆盖表；填两张基准的初值也用它）；点了 --item 时带 --subset（入口没有行、成员没被罩住两样本次未跑）。
#     「行不全」或 --subset 时覆盖格只判看得见的那几样（登记表外的入口、写法不对），别的记本次未跑；
#     两张基准里还写着「# 初值：未填」时 ignore、0 次两格本次未跑并打这一趟的候选行（初值等主 agent 跑一趟不指定范围的门禁之后填）。
#   必红世界、旋钮两格只读树（登记表与变异表）。
#   crash-case-provenance 格（不属出口 4 那五格）同样只读树，起 scripts/crash/crash-case-check.py --only provenance：崩溃点重放合表（.claude/gate.d/crash-point-replay-items.tsv）每一项里，
#     崩溃枚举用例的测试目标调 enumerate_crash_states 的，来历表 crates/singlefs-verification/src/crash_point_replay_provenance.rs 里要有这一项的 enumeration_of_the_item 行；
#     第三列点名的每个放量分项要有 amplification_of_the_item 行。缺的逐条列、判红，红句原样转打；来历表或合表不在记本次未跑。
#   每格的结局并进逐格汇总（「<格名>：绿 / 红 / 本次未跑（为什么）」），本次未跑的逐格往 $GATE_NOT_RUN_FILE 记一行；有一格红整道红。
#
# 复用：按条复用那几趟里每条测试函数各是一条判定，键是它的单元键（测试函数的单元闭包连同闭包真引到的 cfg 原子与这一组特性下的真值、release 档、工具链、解析出的依赖版本，
#   判法 scripts/gate/lib_libtest_result_lines.py 的 recordable_results；算法在 scripts/gate/admission.py 文件头「单元键」），只跑键对不上的那几条
#   （键相同的经 cargo test -- --exact --skip <名> 跳过）。多机配置开没开 GPU 不进键，只是跑的参数（D13 已定项 17：判定不与硬件、不与 CPU 还是 GPU 绑定）。
#   几趟共用 prune-server 那一份判定库里同一个种类（unit-verdicts 族，种类 singlefs-unit-verdicts/<阶段文件名>:test-functions，库名固定、库根不进键，
#   admission.py 文件头「判定库」；D13「每类判定一份库」）：闭包不提 gpu 的测试两趟键相同，
#   不带特性那一趟判绿记下的，带特性那一趟照键复用、不再现判。种类与交给 test-plan 的请求、--skip 的名单、判绿记库的名单只在 scripts/gate/admission.py 的
#   test-functions-request、plan-skip-names、plan-green-names（它文件头「按条复用的共用段」）；scripts/crash/crash-point-replay.sh 按项跑的那一段调同几条、记进同一份库：
#   它写进的判定这里照键复用，这里写进的它照键复用。
#   登记的崩溃枚举用例与放量分项不在按条复用的几趟里：它们的用例函数（登记的 crash-case: 行 test= 的第三段，与放量入口 scripts/crash/crash-amplification.sh 的 ITEMS 里的用例函数，
#   后者经 scripts/crash/lib_crash_point_replay_items.py 的 pipeline_items 读）从 test-plan 的逐条行里去掉，验证包那两趟再经 --skip 排在最前面，交按块那一路。
#   结局经 lib_libtest_result_lines.py 的 recordable_results 逐条认（判法写在它文件头「按条记库」）、判绿的照键记库，按测试函数不记：打了 NOT_RUN_ 的行
#   认作谁打的，那一条不记；判段判在 CPU 的行只在多机配置开了 GPU 时让那一条不记（那一趟判红，同一趟的判定照记：见「多机配置开了 GPU」），没开 GPU 时判在 CPU 的绿照键记库
#   （D13 已定项 17：复用与剪枝不看设备）；结果行被打散的，接得上结局的照记、认不出的那几条不记；完整性闸（没有汇总行、认出的比汇总行报的多）照留，不记的下一趟照跑。
#   记红、撤绿、下趟现判：按条复用的那几趟 cargo test 退非 0 时，report_red_test_targets 抄出的 FAILED 测试函数（结果行、打散后单独一行的结局、libtest 末尾 failures: 名单）
#   按（目标根文件，libtest 全名）对到计划里的那一行，经 admission.py verdict-record --red-names 照计划里的键记红、撤掉同一把键原有的绿（record_red_tests），
#   下一趟这几条现判、原因以「上次判红」起头；scripts/crash/crash-point-replay.sh 按项跑的那一段同一规矩、记进同一份库。
#   有红的那一趟里判绿的照样记绿：同一份输出照上面逐条认（同一套 recordable_results 与完整性闸），认出 ok 的经 plan-green-names 取名单、与记红交同一次
#   verdict-record（record_red_tests 带清单文件），一条别人的红不让同一趟别的绿下一趟重判；认不成、结局认不出、编不过（一条结局行都没有）的一条都不记，下一趟照跑。
#   这一格的结论与出路照旧判红。
#   一条都不用现跑就不起 cargo。拒算（单元闭包收不全）的那一条照跑、不复用、不入库，成功句报拒算条数与头几处形态。
#   成功句首「按条复用」：现判几条、复用几条、拒算几条（scripts/gate/merge-and-verify.sh 读它）。
#   按条复用失灵（test-plan 退非 0、退 0 而没打 PLAN、它或 verdict-record 的复用哨兵报「复用失灵」「判定不稳」「强制现判没起作用」）：判法只在
#   scripts/gate/admission.py 的 plan-failure（文件头「按条复用失灵」），按条复用的几格把那一趟的输出交给它；它判失灵，那一格照旧全部现判跑完、结论判红。
#   崩溃枚举用例的判定单位是崩溃状态块，块键与库写在 crates/singlefs-verification/src/crash_identity.rs 文件头；用例跑完打一行
#   CRASH_CASE_BLOCKS blocks=<块数> reused=<复用> fresh=<现判> red=<红> refused=<拒算>（接口原样写在 scripts/gate/admission.py 文件头「崩溃枚举用例的复用不在这里」）。
#   用例逐条按块跑（run_registered_crash_cases）：照登记表的次序，起 crash-case-command 交的命令（判定在 prune-server 里那一份库，引擎经客户端读写，不设库的变量），
#   块照键复用、键对不上的当场判；日志判法在 admission.py crash-case-judge（test result 恰好 1 passed、登记的计数行、线程），块那一行在 admission.py crash-case-blocks（有红判红）。
#   登记行第三列带 red-blocks=judged-by-test 的用例（故障注入之后的红块由用例自己认下；admission.py crash-case-judged-by-test 答）：cargo test 退非 0 判红，退 0 的块那一行
#   带 --ran 只核在不在、数对不对得上（blocks = reused + fresh + refused），库里有红块也不判红；没登记的用例块那一行有红照旧判红。
#   线程数 SINGLEFS_CRASH_POINT_REPLAY_THREADS 没设取每台默认重活线程数（.claude/heavy-thread-default 的登记值，admission.py 的 crash_case_worker_threads 经 registered_heavy_threads 取）；
#   只起 1 个线程怎么判红，判法在 admission.py crash-case-judge。
#   每条用例都在这一道里单机跑，不在这一件任务里面调分片驱动、不自己连第二台（records/2026-10-30-编译绕过内存总上限.md 收尾批 GT02）。登记了 shard=across-machines 的用例要两台分着判，
#   另经 task-server/task submit -- bash scripts/crash/crash-point-replay-shard-run.sh --merged-log … 交服务按分片协议拆（它文件头「经 task-server 分片」），
#   判过的块落进 prune-server 里同一份判定库，这一道下一趟照块判定库复用。
#   日志：每趟一个新目录 <common-dir>/singlefs-crash-point-replay-logs/<开跑时刻>-<pid>/，跑完不删；log.<用例名> 是用例日志，同名加 .findings.tsv 是发现日志
#   （行格式以 crates/singlefs-verification/src/crash_point_replay_progress.rs 为准）。发现日志判红：某节没有 summary、red_states 不是 0、定稿行数与 signatures= 对不上，
#   或发现日志空而用例日志里有 CRASH_POINT_REPLAY_FINDINGS 行。发现表超过 30 个签名只打前 30 行。
#   不指定范围（没给 --check，也没给 --item）而且没缩过域判绿的这一趟，在日志目录里留一份 all-green（只记这一趟绿过，scripts/gate/apply-writer-patch.py 数「上次崩溃验证全绿以来打了几个补丁」用；不是复用的标记）。
#   缩过域的那一趟（环境里设了 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 或 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO，引擎只往小缩、大于它的常量的不放宽；值是不是非负整数由调用方的入口核）判绿也不写 all-green，
#   成功句写「缩过域、不算门禁通过」并逐个写出设了的变量与值；点没点范围都一样。
#   没有用例级复用标记，不比树、不比 HEAD，不问 stage-must-run.sh 与 change-touches-crates.sh；stage-inputs.tsv 本阶段那一行只登记前提与入口，不进任何键。
#
# 用法（参数顺序不限）：
#   bash .claude/gate.d/crash-point-replay.sh [项目根]                          九格都跑；gate.sh 只传项目根
#   --list                     打格名表（格名、制表符、判什么），不起 cargo、不起 python
#   --check <格名>[,<格名>…]   只跑点名的格
#   --list-items               逐行打可点名的项，不起 cargo：被判那棵树 stage-inputs.tsv 里键是 crash-case: 的行与它们的 test=（「crash-case:<用例名><TAB><test=>」），
#                              再是 验证包 tests/ 里挂特性的测试文件（list_feature_gated_test_files 现读，起一次 python）：「feature-gated:lib<TAB>--lib」
#                              与每个文件一行「feature-gated:<文件名><TAB><whole-file|items> <特性,特性>」；
#                              另带 --with-keys 时末尾再打一行 crash-case-replay 那一格键对不上的测试函数条数（跑 admission.py test-plan，不起 cargo），
#                              再逐条打拒算的测试函数与理由「crash-case-replay<TAB>拒算<TAB><测试函数><TAB><理由>」（一条不落，不只列头几处），
#                              再每项一行 ITEM_PLAN（上面「逐项点名」）；可与 --item 合用（只给点名的项打 ITEM_PLAN，项写错退 2），--check 与项目根照收；
#                              只供证红：GATE_CRASH_POINT_REPLAY_LIST_PLAN_OUTPUT=<文件> 设了就不跑 crash-case-replay 那一条 test-plan，拿这份文件当它的标准输出
#   --item <项>                点名项（上面「逐项点名」；给几次取并集）：crash-case:<用例名>（crash-case: 可写可不写）只限定按块那一路跑哪几条崩溃枚举用例，
#                              feature-gated:<测试文件名> 与 feature-gated:lib 只限定 feature-gated-tests 格编、跑哪几个测试目标
# 退出码：
#   0   绿；带特性那一格有本次未跑时也退 0，逐条报「本次未跑」
#   1   红；有一格红就退 1
#   2   用法错：参数认不出，格名或项写错或缺（含点了 feature-gated 的项而 --check 里没有 feature-gated-tests），项目根进不去，
#       样本档标记目录里的假 cargo 或假 rustc 不在或不可执行，PATH 上的 cargo 没解到标记目录里那一份
#   3   样本档流程判绿：样本档没跑过真的崩溃点重放，不与真跑的绿混在一起
#   75  prune-server 不可用：几格的 test-plan 或 verdict-record 退 75（plan-failure 判不可用，scripts/gate/admission.py 文件头「按条复用失灵」），
#       或备覆盖行时 crash-flow-coverage.py collect 退 75（覆盖、ignore、0 次三格都记不可用），别的格没有红；
#       退 75 的那一格就此收尾（计划那一步退 75 的不起 cargo），别的格照跑；有一格真红就退 1，红的与不可用的两样都报（不是红，也不算通过），
#       出路 python3 prune-server/control check、报给主 agent
#   77  本次未跑：没有 验证包
#   78  准入与运行条件没满足（preflight）
# 不用共用库 lib/stage-cells.sh 跑格：它的汇总把 0、77 之外的退出码并成 1（样本档要退 3）；只 source 它取「弄坏开关只在样本根上认」那一个函数，
# 环境里设了 *_BREAK 而根不是样本根（没有 .gate-cells / .gate-break-sample、或是本仓、--staged 的临时树）就退 2、什么都不跑（--list 与 --list-items 在那之前，不判）。
# 解析参数的判法与共用库同一套（写错退 2、列出格名）。
# 样本只跑点名的格（.claude/rules/verification.md「门禁的结构」，判据同共用库的 stage_cells_apply_sample_marker）：没给 --check、被判的根里有 .gate-cells
#   （一行一个格名，# 之后与空白不算）、这个根（pwd -P）不是本阶段所在的仓、也没设 GATE_STAGED_FROM 时，只跑文件里点名的格；点了没登记的格、一格都没点退 2。
#   本阶段所在的仓与 --staged 的临时树里有这个文件不认，打一行照跑九格。弄坏开关 sample-marker-ignored 下一律不认（改前的样子）。
#
# 覆盖：两条流的崩溃点重放由登记的崩溃枚举用例跑，新池新建文件与 E142 产物逐字比对、「覆盖写、释放、回退与复用」固定脚本到 E 与闭式比对，都由用例自己断言；
#   只做过 mkfs 的池可写挂载再发第一个文件版本那条流与新池新建文件逐项相同，由 cargo test 里的用例钉住，不另枚举。
#   登记的用例不标 ignored，也不在按条复用那一趟里跑：逐条按块跑；用例的判别力在用例自己（计数断言与同文件的靶向阳性对照）。
#
# feature-gated-tests 格（C604（不开特性，崩溃放量与 GPU 对拍编进来零条））：特性从 验证包 tests/ 顶层每个 .rs 的 cfg 现读，不另抄名单；
#   挂特性的文件跑到加复用 0 条、每处 NOT_RUN_NO_GPU_ADAPTER（GPU 对拍没卡时打这一行后返回成功）都报「本次未跑」、不判红；一个挂特性的文件都没有，记无对象可判。
#   清掉调用方的 SINGLEFS_CRASH_AMPLIFICATION_* 与 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE：这一趟不写真的判定库、不按放量的规模跑。
#   键与跑的环境同一份：这一趟（与产品 GPU 真显卡那一趟）的 test-functions-request 也在清掉那些变量的环境里起（crash_point_replay_test_functions_request 用 env 带着 cargo 前面那一串 -u），
#   admission.py 的 DOMAIN_ENVIRONMENT_VARIABLES 按那一刻的环境把改判定域的变量写进键，清掉的变量在键里写成没设；不然键按没生效的变量算（门禁自己没清的环境），
#   不带特性那一趟按调用方的值记下的判绿，带特性那一趟（cargo 实际没看见这个变量）照键复用，跑的和记的不是同一个域。
#   特性里有 gpu 时这一趟要上卡：开跑前核环境里有 task-server 交的 GPU_CARDS_BY_PRIORITY，交给 cargo 的 env 里照它设（文件头「显卡与清场」）；没有就判红、这一趟不起。
#   特性里有 gpu 时这一趟的测试参数带 `-- --nocapture --test-threads 1`（GPU 对拍的测试各占卡的显存，不并行起）；特性里没有 gpu 的不加。
#   例外是 GPU_TEST_THREADS_RELEASED_TARGETS 里的测试目标（今天是 crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state；用户 2026-10-28 定案「放开并按显存排队」，
#   .claude/rules/verification.md 里 GPU 那一条的例外）：特性里有 gpu 时从这一趟拿出来，紧接着另起一趟 cargo test（同一张卡、同一组特性与跳过的测试函数），
#   带 `-- --nocapture --test-threads <每台默认重活线程数>`（scripts/hosts/heavy-threads.sh 的 heavy_thread_default；它的测试按显存额度排队，排队在 crates 里）；
#   两趟的输出并成一份照读（计数、没卡没跑、判在 CPU、红的目标），逐条认结局分开认：带 1 的那一趟带 --serial，另起的那一趟不带，认出的行并进同一份清单。
#   样本 features-gpu-test-threads-released 钉这一条。
#   这个参数只管同时起几个测试，不管一个测试里 CPU 判器自己开的线程：GPU 对拍的测试读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，没设取本机可用并行度
#   （available_parallelism，不照 SINGLEFS_THREAD_CAP 与登记值；改成取登记值归代码批）。样本里的假 cargo 把收到的参数打成一行 CRASH_POINT_REPLAY_PROGRESS sample-cargo-arguments: …，阶段转出来，features-green 的 expect 核这一行带不带 --test-threads 1。
#   --test-threads 1 下 libtest 先打「test <名> ... 」、测试的输出接在后面，结果行常被打散：结局单独起一行的接上照记，认不出的那几条不记（下一趟照跑），不算红；
#   这一趟的记号（NOT_RUN_、判段判在 CPU）按 --serial 认作正在跑的那一条（lib_libtest_result_lines.py 文件头「按条记库」）。
#
# product-gpu-real-card-tests 格（产品的后台 GPU 包 singlefs-background-gpu 的真显卡对拍；欠账出处 records/2026-10-23-subagent拆分提案.md 第一百二十九节「产品 GPU 包的真卡对拍没有门禁在跑」）：
#   对拍测试目标靠特性 tests-on-a-real-gpu 才编（Cargo.toml 的 [[test]] required-features，或测试文件里 cfg 写了 feature = "tests-on-a-real-gpu"），
#   不开这个特性的 cargo test 整包与整工作区都不碰卡，所以 feature-gated-tests 格（只认 验证包 tests/ 里 cfg 点到的特性）认不到它们。
#   这一格按包与特性认（PRODUCT_GPU_PACKAGE_NAME、PRODUCT_GPU_FEATURE_NAME）、不按文件名写死：product_gpu_target_rows 从包目录现读哪几个测试目标挂这个特性，别的实现员以后再加的对拍测试目标也自动进来。
#   一个目标都没有（包不在、包有特性而没有目标挂它）记无对象可判（往 $GATE_NOT_RUN_FILE 记一行，不判红）；包在而 [features] 里没有这个特性判红（特性名对不上了，三处一起改：本阶段、
#   lib_project_code_runs.py 的 PRODUCT_GPU_FEATURE、crates-mutation-rows.py 的 GPU_TEST_FEATURES）。
#   起法：cargo test --release --no-fail-fast -p singlefs-background-gpu --features tests-on-a-real-gpu --test <目标>… -- --nocapture --test-threads 1（不带 --lib：库单测不上卡，归不带特性那一趟），
#   上 task-server 交的卡（与 feature-gated-tests 带 gpu 特性那一趟同一套，require_service_card；环境里没有那张卡就判红、这一趟不起）。
#   对拍测试的 CPU 一侧读 SINGLEFS_CRASH_JUDGE_CPU_THREADS（没设取本机可用并行度），这个参数只管同时起几个测试。
#   按条复用：每条对拍测试函数各一条判定，种类同 验证包那两趟（<阶段文件名>:test-functions，同一份库，admission.py test-plan 的包参数取产品 GPU 包），键相同的经 --exact --skip 跳过，
#   一条都不用现判就不起 cargo；判绿的照键记库，打了 NOT_RUN_ 的那一条不记（lib_libtest_result_lines.py 的 recordable_results）。复用三问（reuse-three-questions）在三份样本里问：
#   product-gpu-reuse-same-inputs（同一份输入再跑一趟：现判 0）、product-gpu-reuse-unrelated-change（改闭包外的输入：现判 0）、product-gpu-reuse-unit-change（改闭包里一个函数单元：现判的恰好是引它的那一条）。
#   判红：对拍那一趟 cargo test 退非 0（有卡的机器上 GPU 核与 CPU 参照对不上；抄出 FAILED 的测试函数与 panic 段，同 report_red_test_targets）；
#   红的照键记红，同一趟判绿的照键记绿（同 验证包那两趟，上面「复用」一段「有红的那一趟里判绿的照样记绿」）。
#   本次未跑（不判红、不记库）：没卡时对拍打的 NOT_RUN_NO_GPU_ADAPTER 逐条报一行（按测试函数）；一个对拍目标跑到加复用 0 条也报。
#   和 crash-case-replay、feature-gated-tests 互不依赖：这一格的行不进覆盖、ignore、0 次那几格。
#   同一道阶段里放的原因：提交时这一道已经要起 cargo 与上卡（commit-stages.md），项目代码闸按阶段文件名一并认（lib_project_code_runs.py 的 STAGE_KIND），不另立阶段文件。
# 按条复用两趟判红时抄出红的测试（report_red_test_targets；crash-case-replay 格不带特性那一趟与 feature-gated-tests 格带特性那一趟 cargo test 退非 0 时）：cargo 的整份输出在每一趟输出目录里
#   （下一段「每一趟当场写结局」），尾巴 40 行常落在最后一个绿的测试目标里，所以另按测试二进制切段（切法与结果行、汇总行的认法取 lib_libtest_result_lines.py），
#   逐个红的目标（汇总行报失败、认出了 FAILED 的结果行，或没有汇总行）打它的 FAILED 测试函数名（结果行、--nocapture 下打散了单独一行的结局、libtest 末尾的 failures: 名单三处取并集）
#   与它的 panic 段（「thread '…' panicked at」起到下一条 test 行、---- 行、failures: 或单独一行的结局之前，一段至多 40 行、一个目标至多 8 段；没有汇总行的另抄末 15 行）；
#   再把整份输出拷到 ${TMPDIR:-/tmp}/crash-point-replay-<格名>.XXXXXX.log 留着、打出路径（样本档收尾核它还在、核完删掉）。
#   只打这些，不改这几格的判定与出路。
# 每一趟当场写结局（records/2026-10-23-subagent拆分提案.md 第一百四十三节「门禁一格里前一趟的结局等整格收尾才写」：feature-gated-tests 格经内存包装起，
#   放开的那一趟撞顶被杀，外层日志只有两趟起跑的标题行，带特性那一趟绿没绿、红在哪一条没有一行，整份输出在草稿目录里跟着没了）：
#   一格里分几趟 cargo 的，每一趟 cargo 一收尾就由 report_pass_outcome 往标准输出打一行这一趟的结局，不等整格收尾：
#   「· <格名>：<哪一趟>收尾，当场写结局：cargo 退 <码>；test result 汇总行 <几> 行（合计通过 / 失败 / ignored；FAILED 的汇总行几行）；FAILED 的测试函数 <几> 条：<名字>…；整份输出：<路径>」。
#   打的趟：crash-case-replay 格不带特性那一趟、每条登记的崩溃枚举用例那一趟；feature-gated-tests 格带特性那一趟、放开 --test-threads 1 的另起那一趟
#   （放开的那一趟起跑之前，带特性那一趟的结局已经打了）。
#   整份输出的路径：按条复用的几趟写在每一趟输出目录 ${TMPDIR:-/tmp}/crash-point-replay-passes.XXXXXX/ 里（草稿目录之外：被 TERM 停时 bash 照样走 EXIT trap 删草稿目录，
#   这一份不跟着删）；每条用例那一趟写在跑完不删的 <common-dir>/singlefs-crash-point-replay-logs/ 下。每一趟输出目录这一道判绿收尾（退 0、样本档退 3）时 EXIT trap 删掉，
#   别的（判红、被打断或被杀）留着、打一行路径，看完自己删。经 scripts/hosts/run-with-memory-cap.sh 起的，TMPDIR 是包装给的那一份：包装退非 0 时整份留着、
#   stderr 打出路径（它文件头「这一条命令自己的临时目录」），撞顶被杀之后照那一行找这一份。样本档收尾一律删掉。
#   弄坏开关 pass-outcome-at-cell-end 下照改前等整格收尾才写（flush_deferred_pass_outcomes）：样本 killed-pass-keeps-earlier-outcome 判错。
#
# 显卡与清场（records/2026-10-30-编译绕过内存总上限.md 收尾批 M03「统一收起来」）：这一道不自己清场、不挑卡。task-server 按名字认出这道门禁要卡
#   （task-server/task_server.py 的 recognize_gpu），套 scripts/crash/gpu-quiesce-run.sh 清场、挑好卡经环境 GPU_CARDS_BY_PRIORITY=<HOSTS 里的地址>:<卡号>:<额度> 交进来。
#   要上卡的两处（feature-gated-tests 带 gpu 特性那一趟、product-gpu-real-card-tests 的对拍那一趟）开跑前都经 require_service_card 核环境里有这张卡：
#   有就打一行用的是哪张卡，自己起的两趟 cargo 在 env 里照它设 GPU_CARDS_BY_PRIORITY；
#   没有就那一格判红、那一趟不起，出路是经 task-server/task submit 交这道门禁。样本档里这张卡只从 .crash-point-replay-sample-tools/caller-environment 来。
#   登记的崩溃枚举用例的卡在 crash-case-command 交的命令里（admission.py 照调用方环境交那一份 GPU_CARDS_BY_PRIORITY），这里不另核。
# 多机配置开了 GPU（ENABLE_GPU=1）：按条复用的几趟都在这份配置下跑，不靠仓根配置关 GPU。配置照 scripts/crash/crash-amplification-lib.sh 的
#   crash_amplification_multi_host_configuration_file 找（${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，样本档只认样本根
#   .crash-point-replay-sample-tools/multi-host.env），找到的那一份在就经 SINGLEFS_MULTI_HOST_CONFIG=<它的绝对路径> 交给两趟 cargo，用例与这里判的是同一份。
#   样本档一进来就 unset 调用方环境里的 SINGLEFS_MULTI_HOST_CONFIG（经 task-server 交时它指那一台的真配置，ENABLE_GPU=1），样本里「调用方带来的配置」只从
#   caller-environment 来，判错的样本不随派到哪一台变；样本档里起准入模块 crash-case-command（定用例的命令带不带 gpu 特性）时再 -u 掉 caller-environment 设的那一份，
#   照它自己的找法读样本根的 multi-host.env：不 -u 的话带来配置的样本也带上 gpu（read_crash_case_command）。真跑照旧读调用方给的那一份。
#   ENABLE_GPU 的值不进任何键（文件头「复用」）：同一条测试在开不开 GPU 下键相同，开没开只决定这一趟怎么跑、判段落在哪。
#   不带特性那一趟（crash-case-replay 格）：编不进 gpu 的用例遇到 ENABLE_GPU=1 打一行 NOT_RUN_<原因> test=<用例> reason=<一句> 报本次未跑、照常返回成功
#     （crates 那一侧的约定；GPU 对拍没卡时的 NOT_RUN_NO_GPU_ADAPTER 是同一种写法）。打了这一行的测试函数（lib_libtest_result_lines.py 认作谁打的）由带特性那一趟补判：
#     那一趟另带上它所在的测试目标（--test <名>、--bin <名>，库那一段本来就在；不论文件里写没写 cfg），它在那一趟判绿、或那一组特性下有键相同的判定复用，就算判到；
#     补判之后仍没判到的（带特性那一格没跑、判红、没有挂特性的文件，认不出是哪一条打的，那一趟也没判绿）才逐条往 $GATE_NOT_RUN_FILE 报一行本次未跑
#     （不判红、不算通过，逐格汇总里 crash-case-replay 格记「不带特性那一趟有 N 处本次未跑」）。日志里有判段判在 CPU 的行
#     （崩溃枚举那一类的 CRASH_POINT_REPLAY_JUDGE route=cpu，或 CRASH_POINT_REPLAY_PARALLEL_START / _FINISHED 带 judge=cpu）判红：配了 GPU 不退回 CPU。
#   带特性那一趟（feature-gated-tests 格）：特性里有 gpu 时上 task-server 交的卡；配置写了 ENABLE_GPU=1 而日志里有判段判在 CPU 的行
#     （同上两种）判红：配了 GPU、又编进了 gpu，判段必须在 GPU。
#   这两趟判段判在 CPU 判红时（cargo test 退 0），同一趟的判定照记（record_cpu_judging_pass，记法同 cargo test 退非 0 那一趟：红的与绿的交同一次 verdict-record）：
#     逐条认结局照「复用」那一段（开了 GPU，判在 CPU 的记号认作谁打的，那几条不记绿）；记号 complete、认作谁打的恰好一条的（带 --test-threads 1 的一段是打它时
#     正在跑的那一条，几条同时跑的一段是打它时只剩那一条还没出结果），归到那一条测试函数，照键记红、撤绿，下一趟现判；别的（几条同时在跑、这一段认不全）
#     只归到它所在的测试二进制，那个二进制里判绿的这一趟一条都不记；别的二进制判绿的照键记绿。
#     样本 gpu-configured-cpu-judged-records-the-rest（不带特性那一趟）、features-gpu-configured-cpu-judged-records-the-rest（带 gpu 特性那一趟）钉这一条。
#   两种记号在一行里哪里都认（前面是行首或空白，cpu_judging_lines）：带 --test-threads 1 的那一趟，测试自己打的头一行接在「test <名> ... 」同一行后面，
#     记号归到那一条测试函数（同一行里的 test 名就是它）照键记红；样本 features-gpu-configured-cpu-judged-on-test-line 钉这一条。
#   判得出的只有崩溃枚举那一类打的这两种行；给定状态与放量流水线的用例有的另写一份 ENABLE_GPU=0 的配置（GPU 对拍的 CPU 那一侧），它们的 mode= 不看。
#   配置没写 ENABLE_GPU=1（关着、没写、文件不在）两趟都不判判段在哪，判段判在 CPU 的那一条判绿照键记库（recordable 带 --cpu-judging-recordable）。
# 按条复用两趟按测试目标并行起（records/2026-10-23-subagent拆分提案.md 第九十七节 ①：cargo test 一个测试二进制接一个地跑，用例少的二进制跑的时候核空着，
#   看门狗报在用核数过少）：不带特性那一趟与特性里没有 gpu 的带特性那一趟，交给 scripts/gate/test-targets-parallel.py --jobs <每台默认重活线程数>
#   （scripts/hosts/heavy-threads.sh 的 heavy_thread_default，与 capped.sh default 取的是同一个数）：先 --no-run 一次编好，再按测试目标各起一条 cargo test、
#   同时至多那么多条；第二个 `--` 之后的 --nocapture 与按条复用的 --exact --skip <名>…
#   原样交给每一条（它文件头「测试二进制的参数」，自证「测试二进制的参数原样交给每个目标」那一格核）。各目标的输出按派活的次序回读进同一份日志，
#   每段照旧以 cargo 的「Running …」起头，下面按条记库、红的测试目标与 NOT_RUN_ 的读法不变；末尾多一行 TEST_TARGETS_PARALLEL …。
#   CRASH_POINT_REPLAY_PROGRESS 行要等全部目标跑完回读时才转出来（不再边跑边打）。
#   照旧一条 cargo test 的：样本档（假 cargo 不认 metadata 与 --no-run）、那份脚本或每台默认重活线程数取不到、带特性那一趟特性里有 gpu
#   （GPU 测试二进制一个接一个、各带 --test-threads 1 上同一张卡，几个二进制同时上卡不在 .claude/rules/verification.md 写的起法里）。
# 样本档：样本根上有 .crash-point-replay-sample-tools/ 就进样本档，把它放到 PATH 最前面（里面的假 cargo、假 rustc 照 cases/ 打日志）：
#   不带 --features 的整包那一趟（--lib --tests）打 cases/default-pass.log，带了打 cases/feature-pass.<--features 的值，逗号换成 +>.log、退出码照同名 .exit（没有那一份日志退 101）；
#   登记的崩溃枚举用例那一趟（不带 --lib、最后一个 --test 的目标有 cases/<目标>.log）打那一份日志、退出码照同名 .exit，设了 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 时把
#   cases/<目标>.findings.tsv 追加进去；登记了 features= 的用例没有 cases/<目标>.log 时落到带特性的那份日志上。样本不放按条复用要判的测试时，计划是空的、不起整包那一趟。
#   样本根另放 .unit-verdict-sample/prior/（上一版的源码树，setup.sh 照这一版拷一份再改）时，开跑前先拿那一版算键、照「判绿」记进这一版的判定库
#   （admission.py verdict-plan / verdict-record：在上一版的树上算、记进同一份库，库名固定），再照常判：
#   expect 读出来的「现判 … 复用 …」逐条点名就是这一版相对上一版该作废与不该作废的测试函数。另放 .unit-verdict-sample/prior-multi-host.env 的，
#   上一版那一趟照它起（它是上一版那一趟的多机配置；GPU 开没开不进键，只有弄坏开关 gpu-switch-in-key 下它的 ENABLE_GPU 进键）。
#   样本由 .claude/singlefs-ai-sop/scripts/stage-selftest.sh 跑，setup.sh 在临时目录里 git init；进库的假工具 git 模式是 100755。
#   样本档在临时 prune-server 底下起（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service）：样本的判定另占临时服务里一份命名空间。
#   样本根的 .crash-point-replay-sample-tools/caller-environment（一行一个「名字=值」，# 起头的行不算）进样本档时当作调用方的环境导出：样本借它模拟调用方设了改判定域的变量。
#   标记目录里放一个 collect-unreachable（内容不读）：备覆盖行那一步的 collect 连一处连不上的服务（socket 不在、TCP 端口刚放掉、重连总时长 2 秒），
#     造「存取覆盖行时 prune-server 不可用」（collect_flow_pass）；按条复用的 test-plan / verdict-record 照常连服务。
#   判红时留下的日志（按条复用两趟 cargo test 退非 0 时那一趟的整份输出）拷到 ${TMPDIR:-/tmp} 下、出路指那一份，收尾删草稿目录时不跟着删；样本档里收尾
#   （crash_point_replay_exit_trap）另核每一份留下的在草稿目录删掉之后还在、内容与留下那一刻逐字节相同，核完删掉（样本不往 TMPDIR 里留东西）。
# 样本（fixtures/crash-point-replay.sh/，每份样本根的 .gate-cells 点名它算给哪一格，expect 的逐格那一行写到点了的每一格；红样本点了几格的，expect 另写一行「<格名>：红」，
#   code-tooling 的 fixture-claims 格照 .gate-cells 与这一行数红绿）：
#   green、red                      登记的两条与三条崩溃枚举用例逐条按块跑：green 发现日志一节 0 个签名、一条不走发现日志，成功句报块现判 3，不指定范围判绿写 all-green；red 两个签名、没有 summary、32 个签名三条用例判红
#   static-check-red                一条调 enumerate_crash_states 却没登记 crash-case: 的 验证包用例、一份第一行没声明模块的测试文件：开跑前的静态判据判红、不起 cargo
#   sample-cargo-not-executable、rocksdb-entry-refused  假 cargo 不可执行退 2；验证包依赖 rocksdb 而假 cargo 不带 rocksdb-prebuilt.py 认的标记：起 cargo 之前判红退 1、假 cargo 一次都没起
#   registered-case-skipped-in-the-plain-pass  登记的 plain_stream_full 与放量分项 amplified_plain_stream_is_clean 在整包那一趟里经 --skip 排在最前面，按块那一路照跑，判绿
#   expand-up-to-narrowed-green     调用方环境里设了 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6（caller-environment）：按块那一路判绿，成功句写缩过域、不算门禁通过，不写 all-green
#   crash-case-provenance-green、crash-case-provenance-red  合表一项的崩溃枚举用例调 enumerate_crash_states：来历表有这一项的 enumeration_of_the_item 行判绿（样本档退 3），没有判红（退 1）
#   judged-by-test-green、judged-by-test-red、judged-by-test-missing-blocks-line  一条没带标记、一条登记 red-blocks=judged-by-test：带标记的 cargo test 退 0 且块那一行对得上判绿（红块不判）；退 101 判红；退 0 而没有块那一行判红
#   judged-by-test-with-red-blocks-green、judged-by-test-missing-blocks-line-after-the-run、unmarked-red-blocks  同一份判法的另三种：带标记的块那一行 red=2 照样判绿；缺块那一行判红；没带标记的块那一行 red=2 判红
#   killed-pass-keeps-earlier-outcome  每一趟当场写结局：整包那一趟绿，登记的用例那一趟的假 cargo 给起它的阶段发 TERM（退 143），输出里已有整包那一趟的结局、没有「逐格」汇总
#   crash-audit-cells-green、crash-audit-caller-gpu-configuration-green  出口 4 那五格都判绿（覆盖行、计数行来自整包那一趟）；后一份另有 ENABLE_GPU=1 的调用方配置，判法相同
#   crash-audit-cells-red             五格各坏一样：StepKind 多一个成员、多一个按段登记的节点、一对新的评估 0 次、用例没有必红变异、旋钮出处不在：五格判红、整道退 1
#   crash-audit-case-log-green        覆盖行与计数行来自登记的用例的日志（extract），整包那一趟没有要判的测试：覆盖、ignore、0 次三格判绿
#   crash-audit-flow-records-unavailable、crash-audit-flow-records-unavailable-with-red  标记目录另放 collect-unreachable：覆盖、ignore、0 次三格记不可用、不判红，整道退 75；后一份另有一格真红，整道退 1，两样都报
#   gpu-configured-plain-pass-not-run、gpu-configured-plain-pass-judged-on-cpu  ENABLE_GPU=1，整包那一趟：用例打 NOT_RUN_ 记本次未跑；日志里有判段判在 CPU 的行判红
#   gpu-configured-cpu-judged-not-recorded、gpu-configured-cpu-judged-records-the-rest、cpu-judged-green-recorded  判段判在 CPU：开了 GPU 的判红且不记，同一趟别的判定照记；没开 GPU 的判绿照记
#   features-green、features-gpu-configured-green、features-gpu-test-threads-released  带特性那一趟：三个挂特性的文件都跑到了用例；ENABLE_GPU=1 时判段在 GPU；放开 --test-threads 1 的目标另起一趟
#   features-gpu-not-run、features-zero、features-red、features-red-outside-the-tail、features-interrupted、features-card-missing  没卡记本次未跑；跑到 0 条记本次未跑；带特性那一趟退 101 判红（红的目标在尾巴之外照样抄出）；被 TERM 打断判红；环境里没有 task-server 交的卡判红
#   features-gpu-configured-judged-on-cpu、features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line  带 gpu 特性那一趟判段判在 CPU 判红，同一趟的判定照记（记号接在 test 行后面的也认）
#   features-shared-library、features-domain-variable-cleared、features-supplement-not-run、features-red-pass-records-green  两趟一份库；键与跑的环境同一份（清掉改判定域的变量）；补判不带特性那一趟打 NOT_RUN_ 的测试；有红的那一趟里判绿的照记
#   features-item-named-green、features-item-unknown  逐项点名（文件头「逐项点名」，样本根的 .gate-items）：点 feature-gated:store_only.rs 只编、只跑 tests/store_only.rs（不带 --lib，特性照整格那一组），
#                                    它的 ITEM_DONE 与 features-green 整格那一趟里同一项那一行逐字相同（点名单跑一项与它在整格里跑的判定相同）；点 tests/ 里没有的文件退 2、列出可点的项
#   red-outside-the-tail、red-pass-records-green、red-rejudged-next-pass  整包那一趟判红（红的目标在尾巴之外）；记红、撤绿、下趟现判
#   unit-key-samples-green、unit-key-samples-red  按条复用（对照样本 S17、N19、F1、H2、H3）：要现判的恰好是 test_t、test_u、f1_middle、h2_test，复用的是 test_v、f1_first、f1_last、h3_test，假 cargo 照收到的 --skip 只打要现判的结果行；后一份让 test_t 判 FAILED
#   unit-key-plan-exit-red、unit-key-plan-sentinel-red、unit-key-plan-unavailable、unit-key-plan-unavailable-with-red  按条复用失灵：test-plan 退 1、哨兵报复用失灵判红；退 75 判不可用、整道退 75（另有真红时退 1）
#   unit-key-gpu-switch-not-in-key、unit-key-not-run-withheld-per-test  GPU 开没开不进键；按测试函数不记（打了 NOT_RUN_ 的那一条、结果行被打散认不出结局的那几条）
#   product-gpu-green、product-gpu-red、product-gpu-no-card、product-gpu-card-missing、product-gpu-zero-tests、product-gpu-no-target、product-gpu-feature-missing、product-gpu-reuse-same-inputs、product-gpu-reuse-unrelated-change、product-gpu-reuse-unit-change
#                                    产品 GPU 包那一格的样本：三条对拍判绿并记库；判红；没卡逐条记本次未跑；环境里没有卡判红；跑到 0 条；没有目标挂特性；包里没有这个特性判红；复用三问
#   四份 features-* 的 tests/ 里挂特性的写法各有一处：整份 #![cfg(feature)]、折成几行的 #![cfg(all(…))]、逐条 #[cfg(feature)]，
#   与一个只在 // 注释里出现的特性名（读进来就找不到那组特性的日志）。样本里的特性 sample-store、sample-accelerator 是造出来的（验证包只有 gpu 一个特性）。
# 弄坏开关（只给证红用）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/crash-point-replay.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   GATE_CRASH_POINT_REPLAY_BREAK=registered-cases-not-skipped  整包那几趟不把登记的崩溃枚举用例函数排进 --skip（计划里的行照去）
#       registered-case-skipped-in-the-plain-pass（输出里找不到「sample-cargo-arguments: … --exact --skip amplified_plain_stream_is_clean --skip plain_stream_full」）
#   GATE_CRASH_POINT_REPLAY_BREAK=amplification-items-not-skipped  不取放量入口 ITEMS 里的用例函数：整包那几趟不排放量分项
#       registered-case-skipped-in-the-plain-pass（输出里找不到「--skip amplified_plain_stream_is_clean」那一行）
#   GATE_CRASH_POINT_REPLAY_BREAK=all-green-not-written  不指定范围判绿也不写 all-green
#       green（输出里找不到「不指定范围，这一趟判绿：」）
#   GATE_CRASH_POINT_REPLAY_BREAK=narrowed-run-writes-all-green  缩过域（环境里设了展开深度变量）的那一趟判绿也写 all-green、不写缩过域那一句（改前的样子：不看缩域变量）
#       expand-up-to-narrowed-green（输出里找不到「缩过域、不算门禁通过：SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6」）
#   GATE_CRASH_POINT_REPLAY_BREAK=provenance-judge-not-run  crash-case-provenance 格不起 crash-case-check.py 的 provenance 判法、直接记绿
#       crash-case-provenance-red（期望退出 1，实测 3：来历表缺行的合表项没人判，这一格记绿）
#   GATE_CRASH_POINT_REPLAY_BREAK=registered-rows-in-plan  整包那几趟与样本档的上一版算计划时不带 --exclude-registered（登记的崩溃枚举用例进计划、能被挑成复用哨兵、样本档替它记绿）
#       unit-key-samples-green（输出里找不到「上一版（.unit-verdict-sample/prior）的 8 条测试函数」：plain_stream_full 也记了）、
#       unit-key-gpu-switch-not-in-key（输出里找不到「上一版（.unit-verdict-sample/prior）的 3 条测试函数」）、
#       unit-key-plan-sentinel-red（输出里找不到「复用失灵：crates/singlefs-verification/tests/setting.rs::test_t 键相同却现判了」：头一条请求成了 plain_stream.rs 那一条）
#   GATE_CRASH_POINT_REPLAY_BREAK=setting-file-whole-closure  整包那几趟每条测试函数的键按它所在的整份设点文件算（文件里每个单元都进键：改同文件里别的测试也作废）
#       unit-key-samples-green（输出里找不到「复用 crates/singlefs-verification/tests/setting.rs::test_v」：N19 那一格，只改了 test_u 的一个字，同文件的 test_v 也被判成要现判）；
#       unit-key-samples-red（输出里找不到「--skip f1_first --skip f1_last --skip h3_test --skip test_v」：复用的那几条不再 --skip）
#       features-shared-library（输出里找不到「变了 1 处：变了 fn crates/singlefs-verification/src/lib.rs::tests::shared_check）」）
#   GATE_CRASH_POINT_REPLAY_BREAK=plan-unavailable-as-failure  plan-failure 退 75 照按条复用失灵办（这一格照旧全部现判、判红），不退 75（改前的样子）
#       unit-key-plan-unavailable（期望退出 75，实测 1）；unit-key-plan-unavailable-with-red（输出里找不到「✗ crash-case-replay 这一格没判：prune-server 不可用（上面那一句）；这一格就此收尾…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=plan-unavailable-exits-at-once  plan-failure 退 75 当场整道退 75，后面的格不跑、已判红的格被吞（改前的样子）
#       unit-key-plan-unavailable-with-red（期望退出 1，实测 75）；unit-key-plan-unavailable（输出里找不到「✗ crash-case-replay 这一格没判：prune-server 不可用（上面那一句）；这一格就此收尾…」）
#   admission.py 的 ADMISSION_BREAK=plan-failure-not-red  按条复用失灵也不判红（改前整包那一趟退回全部现判、不判红的样子）
#       unit-key-plan-exit-red、unit-key-plan-sentinel-red（期望退出 1，实测 3：流程判绿）
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
#       features-green、features-gpu-not-run、features-red、features-interrupted、features-zero、features-gpu-configured-green、features-supplement-not-run、
#       features-shared-library、features-red-pass-records-green、features-gpu-configured-judged-on-cpu、features-red-outside-the-tail、
#       features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line、cpu-judged-green-recorded、gpu-configured-cpu-judged-not-recorded、
#       gpu-configured-cpu-judged-records-the-rest、red-outside-the-tail、red-pass-records-green、red-rejudged-next-pass（带特性那一趟没起：输出里找不到它的结局句、期望退出 1 实测 3，或补判的结论报本次未跑）；
#       features-item-named-green（输出里找不到「sample-cargo-arguments: … --test store_only …」）；
#       features-domain-variable-cleared、features-gpu-test-threads-released（带特性那一趟没起：输出里找不到它那一趟的 sample-cargo-arguments 行）
#   GATE_CRASH_POINT_REPLAY_BREAK=features-not-from-source  特性不从源码读、写死 sample-store,gpu（假 cargo 没有这组特性的日志、退 101）
#       features-gpu-not-run、features-green、features-zero（期望退出 3，实测 1）；features-interrupted、features-red、features-red-outside-the-tail、red-outside-the-tail、red-rejudged-next-pass、
#       red-pass-records-green、features-red-pass-records-green、features-shared-library、cpu-judged-green-recorded、features-gpu-configured-green、features-item-named-green、
#       features-domain-variable-cleared、features-gpu-configured-cpu-judged-on-test-line、features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-judged-on-cpu、
#       features-gpu-test-threads-released、features-supplement-not-run（带特性那一趟找不到这组特性的日志）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-feature-pass-exit  带特性那一趟退非 0 当成跑过
#       features-interrupted、features-red、features-red-outside-the-tail、features-red-pass-records-green（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-zero-feature-files  不判挂特性的文件跑到 0 条
#       features-zero（输出里找不到「本次未跑（这不是通过）：验证包 crates/s…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-gpu-not-run  不读 NOT_RUN_NO_GPU_ADAPTER
#       features-gpu-not-run（输出里找不到「本次未跑（这不是通过）：GPU 对拍没跑…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=gpu-agreement-test-threads-one  带 gpu 特性那一趟不把 GPU_TEST_THREADS_RELEASED_TARGETS 里的测试目标拿出来另起（照改前一趟、一律带 --test-threads 1）
#       features-gpu-test-threads-released（输出里找不到「--lib --test gpu_pair -- --nocapture --test-threads 1」与「放开 --test-threads 1 的 GPU 测试目标另起一趟」：放开的那个目标并在带 1 的那一趟里）
#   GATE_CRASH_POINT_REPLAY_BREAK=gpu-test-threads-unset  特性里有 gpu 也不加 --test-threads 1（照改前，测试线程取 RUST_TEST_THREADS 即登记值）
#       features-green、features-item-named-green、features-gpu-test-threads-released（输出里找不到「…-- --nocapture --test-threads 1」）；features-supplement-not-run（输出里找不到「sample-cargo-arguments: … --features gpu …」）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-cell-skipped  要现判的也不起产品 GPU 包的对拍那一趟（日志空着、退出码当 0）
#       product-gpu-green、product-gpu-no-card、product-gpu-zero-tests、product-gpu-reuse-unit-change（输出里找不到「sample-cargo-arguments: test --release --no-fail-fast -p singlefs-background-gpu…」）；product-gpu-red（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-test-threads-unset  产品 GPU 包的对拍那一趟不加 --test-threads 1（几个对拍测试各占卡的显存，并行起会互相抢）
#       product-gpu-green、product-gpu-red、product-gpu-no-card、product-gpu-zero-tests、product-gpu-reuse-unit-change（输出里找不到「…-- --nocapture --test-threads 1」）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-exit-ignored  产品 GPU 包的对拍那一趟 cargo test 退非 0 当成跑过
#       product-gpu-red（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-not-run-ignored  不读产品 GPU 包对拍打的 NOT_RUN_NO_GPU_ADAPTER
#       product-gpu-no-card（输出里找不到「本次未跑（这不是通过）：产品 GPU 包真显卡对拍没跑…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-zero-ignored  不判产品 GPU 包的对拍目标跑到 0 条
#       product-gpu-zero-tests（输出里找不到「本次未跑（这不是通过）：产品 GPU 包 crates/singlefs-background-gpu/tests/two_kernels_reconcile.rs 挂特性…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-targets-by-file-name  对拍目标按文件名认（只认 tests/checksum 起头的），不按包与特性认
#       product-gpu-green、product-gpu-no-card、product-gpu-zero-tests、product-gpu-reuse-same-inputs、product-gpu-reuse-unrelated-change、product-gpu-reuse-unit-change、product-gpu-red（cfg 挂特性的 two_kernels_reconcile 认不到）
#   GATE_CRASH_POINT_REPLAY_BREAK=card-missing-unrefused  要上卡的一趟环境里没有 task-server 交的 GPU_CARDS_BY_PRIORITY 也照起（require_service_card 不判红）
#       features-card-missing、product-gpu-card-missing（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=sample-caller-configuration-kept  样本档里准入模块照旧读调用方环境里 SINGLEFS_MULTI_HOST_CONFIG 指的多机配置（read_crash_case_command 不 -u 它；改前的样子）
#       crash-audit-caller-gpu-configuration-green（期望退出 3，实测 1）
#   GATE_CRASH_POINT_REPLAY_BREAK=service-card-not-forwarded  task-server 交的卡不交给这一道自己起的那两趟 cargo（env 里 -u GPU_CARDS_BY_PRIORITY）
#       features-green、features-red、features-interrupted、features-supplement-not-run、product-gpu-green、product-gpu-red、product-gpu-no-card、product-gpu-zero-tests、product-gpu-reuse-unit-change
#       （输出里找不到「sample-cargo-gpu: cards=[192.0.2.1:0:8G] quiesced-by-the-stage=[]」：假 cargo 收到的是 cards=[unset]）
#   GATE_CRASH_POINT_REPLAY_BREAK=stage-quiesces-by-itself  要上卡的一趟照改前自己套 scripts/crash/gpu-quiesce-run.sh（清场归 task-server 之前的样子；清场配置是草稿目录里只写假清场命令的一份）
#       同上九份样本（输出里找不到「sample-cargo-gpu: cards=[192.0.2.1:0:8G] quiesced-by-the-stage=[]」：假 cargo 收到的是 quiesced-by-the-stage=[yes]）
#   GATE_CRASH_POINT_REPLAY_BREAK=red-tail-only  按条复用两趟判红时照改前只打 cargo 输出的尾巴 40 行与「^test … FAILED$」那一条 grep：不抄红的测试目标里 FAILED 的测试函数名与 panic 段、不留整份输出
#       product-gpu-red（输出里找不到「· product-gpu-real-card-tests：红的测试目标 1 个、FAILED 的测试函数 1 条」）；
#       features-red-outside-the-tail（输出里找不到「· feature-gated-tests：红的测试目标 1 个、FAILED 的测试函数 1 条…」）；
#       red-outside-the-tail（输出里找不到「· crash-case-replay：红的测试目标 1 个、FAILED 的测试函数 1 条…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=pass-outcome-at-cell-end  每一趟 cargo 的结局照改前不当场打：先记进草稿目录，整格收尾才打（文件头「每一趟当场写结局」；后一趟被杀时整格收不了尾，前一趟的结局跟着草稿目录没了）
#       killed-pass-keeps-earlier-outcome（输出里找不到「· crash-case-replay：不带特性那一趟收尾，当场写结局：cargo 退 0；…」：登记的用例那一趟把阶段停了，整格没收尾）
#   GATE_CRASH_POINT_REPLAY_BREAK=sample-tools-unchecked  不核标记目录里的假工具
#       sample-cargo-not-executable（期望退出 2，实测 1）
#   GATE_CRASH_POINT_REPLAY_BREAK=rocksdb-entry-unchecked  起 cargo 之前不调 rocksdb-prebuilt.py entry
#       rocksdb-entry-refused（输出里找不到「rocksdb-prebuilt entry：」：往下走起了假 cargo）
#   GATE_CRASH_POINT_REPLAY_BREAK=cpu-judging-accepted  配置写了 ENABLE_GPU=1 也不看判段判在哪
#       gpu-configured-plain-pass-judged-on-cpu、features-gpu-configured-judged-on-cpu、gpu-configured-cpu-judged-not-recorded、
#       gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=plain-pass-not-run-unreported  不带特性那一趟日志里的 NOT_RUN_<原因> 行不收、不报本次未跑
#       gpu-configured-plain-pass-not-run、features-gpu-configured-green（输出里找不到「本次未跑（这不是通过）：验证包不带特性那一趟…」）；
#       unit-key-not-run-withheld-per-test（输出里找不到「验证包 现判 5、复用 0、拒算 0、本次未跑 1…」）；
#       features-supplement-not-run（输出里找不到「· feature-gated-tests：不带特性那一趟打了 NOT_RUN_ 的测试函数由这一趟补判…」：收不到就没东西可补判）
#   GATE_CRASH_POINT_REPLAY_BREAK=list-items-refusals-counted-only  --list-items --with-keys 照改前只报拒算条数、不逐条列拒算的测试函数与理由
#       样本档喂不进 --list-items（stage-selftest.sh 只传根），证红的输入是一份照 test-plan 标准输出的写法造的文件（几条 refused、几条 judge 与 reuse、末行 PLAN）：
#       GATE_CRASH_POINT_REPLAY_LIST_PLAN_OUTPUT=<那份文件> bash .claude/gate.d/crash-point-replay.sh --list-items --with-keys <根>，
#       开关关着逐条打那几行「crash-case-replay<TAB>拒算<TAB>…」，打开时一行都没有
#   GATE_CRASH_POINT_REPLAY_BREAK=judged-by-test-ignores-exit  登记了 red-blocks=judged-by-test 的用例 cargo test 退非 0 也当绿
#       judged-by-test-red（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=judged-by-test-unrecognized  不认 red-blocks=judged-by-test，带标记的用例照没标记办（块那一行有红判红，不带 --ran）
#       judged-by-test-green、judged-by-test-with-red-blocks-green（期望退出 3，实测 1）；judged-by-test-missing-blocks-line、judged-by-test-missing-blocks-line-after-the-run（输出里找不到「crash-case-blocks --ran 退 4」）；
#       unmarked-red-blocks（输出里找不到「✗ 1 条崩溃枚举用例判红：crash-case:plain-stream（这一趟判绿 1 条」：带标记的那条也判红）
#   GATE_CRASH_POINT_REPLAY_BREAK=ran-for-every-case  每条用例块那一行都带 --ran（没登记 red-blocks=judged-by-test 的用例块那一行有红也不判红）
#       unmarked-red-blocks（期望退出 1，实测 3）
#   admission.py 的 ADMISSION_BREAK=ran-blocks-line-optional  crash-case-blocks --ran 没有那一行也退 0
#       judged-by-test-missing-blocks-line、judged-by-test-missing-blocks-line-after-the-run（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=gpu-switch-in-key  按条复用的几趟每条测试函数的键照改前另加 line:enable-gpu=<多机配置的 ENABLE_GPU>（样本档拿上一版算键时取上一版那一趟的配置）
#       unit-key-gpu-switch-not-in-key（输出里找不到「· 按条复用：验证包 3 条编得进的测试函数，现判 0、复用 3…」：上一版在 ENABLE_GPU=0 下记的判定这一版对不上键）
#   GATE_CRASH_POINT_REPLAY_BREAK=withhold-whole-binary  照改前整段不记：一段里有 NOT_RUN_ / 判段判在 CPU 的行、或没打散的结果行条数与汇总行对不上，那一段一条都不记
#       （转成 lib_libtest_result_lines.py 的 LIBTEST_RESULT_LINES_BREAK=marker-withholds-whole-binary,count-mismatch-withholds-whole-binary）
#       unit-key-not-run-withheld-per-test（输出里找不到「· 判绿不记库 2 条（这一趟现判、按条记库不认的；下一趟照跑）：」）；
#       features-supplement-not-run（输出里找不到「· 判绿记库：RECORD recorded=1 changed=0 refused=0」）；
#       gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line（输出里找不到「判段判在 CPU 的记号 2 处：归到测试函数的…」：每处记号都认作整段打的）
#   GATE_CRASH_POINT_REPLAY_BREAK=library-per-pass  照改前几趟各一份库（<阶段文件名>:crash-case-replay 与 <阶段文件名>:feature-gated-tests）
#       features-shared-library、cpu-judged-green-recorded、features-supplement-not-run、red-rejudged-next-pass、red-pass-records-green、features-red-pass-records-green、
#       gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line（带特性那一趟查的是另一份库，复用的数对不上）
#   GATE_CRASH_POINT_REPLAY_BREAK=no-supplement  不补判：带特性那一趟不另带不带特性那一趟打了 NOT_RUN_ 的测试函数所在的测试目标
#       features-supplement-not-run（输出里找不到「· feature-gated-tests：不带特性那一趟打了 NOT_RUN_ 的测试函数由这一趟补判…」：enumeration_default_shape 没跑到、报本次未跑）
#   GATE_CRASH_POINT_REPLAY_BREAK=cpu-judging-withheld  照改前判段判在 CPU 的一律不记：多机配置没开 GPU 也不给 recordable 带 --cpu-judging-recordable
#       cpu-judged-green-recorded（带特性那一趟又现判 tests::cpu_judged_check）
#       （库那一侧同一种弄坏是 lib_libtest_result_lines.py 的 LIBTEST_RESULT_LINES_BREAK=cpu-judging-always-withheld）
#   GATE_CRASH_POINT_REPLAY_BREAK=cpu-judging-recorded-when-gpu-configured  多机配置开了 GPU 也带 --cpu-judging-recordable（判段判在 CPU 的那一条照记）
#       gpu-configured-cpu-judged-not-recorded、gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line（输出里找不到「· 判绿不记库 N 条…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=sample-marker-ignored  样本根的 .gate-cells 不认，九格照跑（改前的样子：不带覆盖、ignore 两张表与 segments.rs 的样本在覆盖、ignore、0 次三格判红）
#       green（期望退出 3，实测 1：出口 4 有格判红）等所有点名格少于九格而退 3 的样本；红样本（features-red、product-gpu-red 等）输出里找不到「✗ 有 1 格判红：…」
#   GATE_CRASH_POINT_REPLAY_BREAK=crash-audit-red-ignored  出口 4 那五格与 crash-case-provenance 红了也不算进整道（不进红格）
#       crash-audit-cells-red（期望退出 1，实测 3）；unit-key-plan-unavailable-with-red（期望退出 1，实测 75：crash-knobs 的红不算，只剩不可用）；
#       crash-audit-flow-records-unavailable（期望退出 75，实测 3：不可用的三格也被清出整道）；crash-audit-flow-records-unavailable-with-red（期望退出 1，实测 3）；crash-case-provenance-red（期望退出 1，实测 3：来历表缺行的红也被清出整道）
#   GATE_CRASH_POINT_REPLAY_BREAK=flow-records-unavailable-as-failure  collect 退 75 照判法脚本出错办（覆盖、ignore、0 次三格判红，改前的样子）
#       crash-audit-flow-records-unavailable（期望退出 75，实测 1）；crash-audit-flow-records-unavailable-with-red（输出里找不到「✗ crash-flow-coverage 这一格没判：prune-server 不可用（…」）
#   static-check 格与出口 4 那几格的判法开关（CRASH_CASE_CHECK_BREAK、CRASH_FLOW_COVERAGE_BREAK）不在这里列：scripts/crash/crash-case-check.py、
#   scripts/crash/crash-flow-coverage.py 平常跑时设了开关整份退 2（只在 --selftest 里认），在样本上证不了；开关表在它们文件头，由它们的 --selftest 证红
#   GATE_CRASH_POINT_REPLAY_BREAK=red-not-recorded  按条复用的几趟判红时不把 FAILED 的测试函数交 verdict-record --red-names（改前的样子：门禁判红的不记红、不撤同一把键原有的绿），判绿的照记
#       red-rejudged-next-pass（输出里找不到「· 判红记库：1 条记红、撤掉同一把键原有的绿，下一趟照现判（RECORD recorded=0 changed=0 refused=0…」）；
#       red-pass-records-green、features-red-pass-records-green、product-gpu-red（输出里找不到「· 判红记库：1 条记红、撤掉同一把键原有的绿，下一趟照现判（RECORD recorded=…」）；
#       gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line（同一句：判段判在 CPU 归得到的那几条没记红）
#   GATE_CRASH_POINT_REPLAY_BREAK=plain-pass-red-withholds-green  crash-case-replay 格不带特性那一趟 cargo test 退非 0 时照改前不认结局、判绿的一条都不记（只记红）
#       red-pass-records-green（输出里找不到「· 判红记库：1 条记红、撤掉同一把键原有的绿，下一趟照现判（RECORD recorded=1 changed=0 refused=0…」：green_check 没记绿）
#   GATE_CRASH_POINT_REPLAY_BREAK=feature-pass-red-withholds-green  feature-gated-tests 格带特性那一趟 cargo test 退非 0 时照改前不认结局、判绿的一条都不记（只记红）
#       features-red-pass-records-green（输出里找不到「· 判红记库：1 条记红、撤掉同一把键原有的绿，下一趟照现判（RECORD recorded=1 changed=0 refused=0…」：store_only_quick 没记绿）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-red-withholds-green  product-gpu-real-card-tests 格对拍那一趟 cargo test 退非 0 时照改前不认结局、判绿的一条都不记（只记红）
#       product-gpu-red（输出里找不到「· 判红记库：1 条记红、撤掉同一把键原有的绿，下一趟照现判（RECORD recorded=2 changed=0 refused=0…」：判绿的另 2 条没记）
#   GATE_CRASH_POINT_REPLAY_BREAK=test-functions-request-gate-environment  test-functions-request 不带 cargo 前面那一串 env -u，按门禁自己没清的环境起（改前的样子：键按没生效的变量算）
#       features-domain-variable-cleared（输出里找不到「新的输入行 variable SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO unset」：带特性那一趟的键还带着变量的 3，shared_check 照键复用，现判只有 1 条）
#   GATE_CRASH_POINT_REPLAY_BREAK=plain-pass-cpu-judging-records-nothing  crash-case-replay 格不带特性那一趟配了 GPU、日志里有判段判在 CPU 的行时照改前一条都不记（不调 record_cpu_judging_pass）
#       gpu-configured-cpu-judged-records-the-rest（输出里找不到「· crash-case-replay：判段判在 CPU 的记号 2 处：…」：cpu_judged_check 没记红、settled_check 与 other_green 没记绿）
#   GATE_CRASH_POINT_REPLAY_BREAK=feature-pass-cpu-judging-records-nothing  feature-gated-tests 格带 gpu 特性那一趟配了 GPU、日志里有判段判在 CPU 的行时照改前一条都不记（不调 record_cpu_judging_pass）
#       features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line（输出里找不到「· feature-gated-tests：判段判在 CPU 的记号 2 处：…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=cpu-judging-line-start-only  cpu_judging_lines 照改前只认去掉行首空白后以记号起头的行（接在「test <名> ... 」同一行后面的认不出）
#       features-gpu-configured-cpu-judged-on-test-line（期望退出 1，实测 3：两处记号都接在 test 名后面，带 gpu 特性那一趟一处判在 CPU 都没认出）
#   逐项点名（文件头「逐项点名」）的四个：
#   GATE_CRASH_POINT_REPLAY_BREAK=feature-items-ignored  点了 feature-gated 的项也照整格跑（--lib 与全部挂特性的文件）
#       features-item-named-green（输出里找不到「· feature-gated-tests：只编、只跑 --item 点名的 1 项：feature-gated:store_only.rs」）
#   GATE_CRASH_POINT_REPLAY_BREAK=feature-item-features-narrowed  点了 feature-gated 的项时特性只取点名的文件的（键与整格的不同，点名单跑与整格判的不是同一份）
#       features-item-named-green（期望退出 3，实测 1：特性只剩 sample-store，假 cargo 没有这组特性的日志）
#   GATE_CRASH_POINT_REPLAY_BREAK=feature-item-unknown-accepted  点了 tests/ 里没有的文件也照收（悄悄丢掉，不退 2）
#       features-item-unknown（期望退出 2，实测 3：丢掉之后一项都没点，照整格跑）
#   GATE_CRASH_POINT_REPLAY_BREAK=item-done-unreported  跑完不打 ITEM_DONE（feature-gated 的项与崩溃枚举用例都不打）
#       features-green、features-item-named-green（输出里找不到「ITEM_DONE item=feature-gated:…」）
crash_point_replay_range_given=0
root_argument=""
crash_point_replay_list_cells=0
crash_point_replay_list_items=0
crash_point_replay_list_with_keys=0
crash_point_replay_named_cases=()
# 格名表：与文件头九行 `# gate-cell:` 逐字相同、次序相同
CRASH_CASE_REPLAY_CELL_NAME="crash-case-replay"
CRASH_CASE_REPLAY_CELL_TITLE="包里全部测试函数按单元键复用（登记的崩溃枚举用例经 --skip 排掉）、登记的崩溃枚举用例逐条按崩溃状态块的单元键跑（块键相同的复用、缺的当场判）"
FEATURE_GATED_TESTS_CELL_NAME="feature-gated-tests"
# 带 gpu 特性那一趟里放开 --test-threads 1 的测试目标（文件头「feature-gated-tests 格」；scripts/crash/crash-point-replay.sh 与 scripts/gate/admission.py 的同名名单是同一份，三处一起改）
GPU_TEST_THREADS_RELEASED_TARGETS=(crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state)
FEATURE_GATED_TESTS_CELL_TITLE="带上 验证包 tests/ 里 cfg 点到的特性另跑一趟（每条测试函数按单元键复用）：挂特性的测试文件逐个跑到或复用的用例不为 0，GPU 对拍没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑"
PRODUCT_GPU_REAL_CARD_CELL_NAME="product-gpu-real-card-tests"
PRODUCT_GPU_REAL_CARD_CELL_TITLE="起产品 GPU 包（singlefs-background-gpu）带 tests-on-a-real-gpu 特性才编的真显卡对拍测试（经清场、--test-threads 1，每条测试函数按单元键复用）：对拍判红整格红，没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑"
# 产品的后台 GPU 包与它的真显卡对拍特性（Cargo.toml 的 [features]；测试目标靠 [[test]] 的 required-features 或测试文件的 cfg 挂在它后面，product_gpu_target_rows 现读）
PRODUCT_GPU_PACKAGE_NAME="singlefs-background-gpu"
PRODUCT_GPU_FEATURE_NAME="tests-on-a-real-gpu"
# --check 点名的格（空着就是九格都跑）
crash_point_replay_selected_cells=()
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# 弄坏开关只在样本根上认（C622）：判法只在共用库一处（lib/stage-cells.sh 的 stage_cells_refuse_unsampled_break_switches，
# 它文件头「弄坏开关只在样本根上认」）。这一道不经共用库跑格，只 source 它取这一个函数（cd 之前 source，库按本阶段的路径认阶段所在的仓），cd 到根之后调。
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
# 出口 4 那五格（文件头「出口 4 那五格」）：名字与判什么，次序同文件头
CRASH_AUDIT_CELL_NAMES=(crash-flow-coverage crash-ignored-nodes invariant-zero-evaluations crash-case-must-red crash-knobs)
CRASH_AUDIT_CELL_TITLES=(
  "崩溃流覆盖表：没有登记表 crash-entries.tsv 外的入口、登记的入口都有覆盖行、StepKind 每个成员都被一条流罩住"
  "ignore 开着的节点（ignored_point、registered_segment、not_expanded）不比基准 crash-ignored-nodes.tsv 多"
  "评估 0 次的（流，不变量）对都在带理由的基准 invariant-zero-evaluations.tsv 里"
  "每条崩溃枚举用例在 crates/mutations.tsv 里至少有一行变异点名它的测试目标"
  "旋钮登记表 crash-knobs.tsv 每行取值与依据不空、出处路径在"
)
# 读这一趟的覆盖行与计数行的三格
CRASH_AUDIT_FLOW_CELLS=(crash-flow-coverage crash-ignored-nodes invariant-zero-evaluations)
# 合表各项的来历表核对（不属出口 4 那五格，判法同样在 crash-case-check.py，只读树）
CRASH_CASE_PROVENANCE_CELL_NAME="crash-case-provenance"
CRASH_CASE_PROVENANCE_CELL_TITLE="崩溃点重放合表里调 enumerate_crash_states 的崩溃枚举项与放量分项，在来历表 crash_point_replay_provenance.rs 里都有一行"
# crash_point_replay_print_cells：逐行打「格名、制表符、判什么」，九格，次序同文件头
crash_point_replay_print_cells() {
  local index
  printf '%s\t%s\n' "$CRASH_CASE_REPLAY_CELL_NAME" "$CRASH_CASE_REPLAY_CELL_TITLE" "$FEATURE_GATED_TESTS_CELL_NAME" "$FEATURE_GATED_TESTS_CELL_TITLE" \
    "$PRODUCT_GPU_REAL_CARD_CELL_NAME" "$PRODUCT_GPU_REAL_CARD_CELL_TITLE"
  for index in "${!CRASH_AUDIT_CELL_NAMES[@]}"; do printf '%s\t%s\n' "${CRASH_AUDIT_CELL_NAMES[index]}" "${CRASH_AUDIT_CELL_TITLES[index]}"; done
  printf '%s\t%s\n' "$CRASH_CASE_PROVENANCE_CELL_NAME" "$CRASH_CASE_PROVENANCE_CELL_TITLE"
}
# crash_point_replay_cell_in <格名> <格名…>：头一个参数在不在后面那一串里
crash_point_replay_cell_in() {
  local wanted="$1" candidate
  shift
  for candidate in "$@"; do [[ "$candidate" == "$wanted" ]] && return 0; done
  return 1
}
crash_point_replay_check_cells() { # <逗号隔开的格名> [<点名的来源，缺省 --check>]：认格名表里的九格，记进 crash_point_replay_selected_cells；写错、空着退 2 并列出可用的格名
  local requested_cell requested_cells=() selection_source="${2:---check}"
  IFS=, read -r -a requested_cells <<< "$1"
  ((${#requested_cells[@]})) || requested_cells=("")
  for requested_cell in "${requested_cells[@]}"; do
    if [[ -z "$requested_cell" ]] || ! crash_point_replay_cell_in "$requested_cell" "$CRASH_CASE_REPLAY_CELL_NAME" "$FEATURE_GATED_TESTS_CELL_NAME" "$PRODUCT_GPU_REAL_CARD_CELL_NAME" "${CRASH_AUDIT_CELL_NAMES[@]}" "$CRASH_CASE_PROVENANCE_CELL_NAME"; then
      echo "  ✗ crash-point-replay.sh：$selection_source 里有没登记的格名：「$requested_cell」"
      echo "     → 怎么办：格名照下面写（也可以 bash .claude/gate.d/crash-point-replay.sh --list 看）；按崩溃枚举用例点名用 --item <用例名>："
      crash_point_replay_print_cells | sed 's/\t/  /; s/^/       /'
      exit 2
    fi
    crash_point_replay_selected_cells+=("$requested_cell")
  done
}
crash_point_replay_runs_cell() { # <格名>：这一趟跑不跑这一格（没给 --check 九格都跑）
  local selected_cell
  ((${#crash_point_replay_selected_cells[@]})) || return 0
  for selected_cell in "${crash_point_replay_selected_cells[@]}"; do [[ "$selected_cell" == "$1" ]] && return 0; done
  return 1
}
# crash_point_replay_test_functions_request <阶段文件名> <判法根>：按条复用的请求（判定种类、交给 test-plan 的参数）只在 scripts/gate/admission.py 的
#   test-functions-request 一处（它文件头「按条复用的共用段」；scripts/crash/crash-point-replay.sh 按项跑的那一段调同一条，两边记进、读的是同一类判定库），
#   放进全局 crash_point_replay_test_functions_kind 与数组 crash_point_replay_test_functions_arguments。读不出时种类写 <阶段文件名>:test-functions-request-unreadable、
#   参数写成一个 test-plan 认不出的 --token（那一趟 test-plan 退 2，照文件头「复用」里「按条复用失灵」判红，不悄悄换一把键）。
#   test-functions-request 在交给 cargo 的同一份环境里起（admission.py 的 DOMAIN_ENVIRONMENT_VARIABLES 按那一刻的环境把改判定域的变量写进键）：带特性那一趟与产品 GPU 真显卡那一趟
#   cargo 前面是 env -u 清掉调用方 SINGLEFS_CRASH_AMPLIFICATION_* 的那一串（cleared_environment），这两格先把那一串放进全局数组 crash_point_replay_test_functions_environment，
#   这里用 env 带着它起 test-functions-request；不带特性那一趟 cargo 不清，数组是空的。弄坏开关 test-functions-request-gate-environment 下不带（改前的样子：按门禁自己没清的环境起）
crash_point_replay_test_functions_environment=()
# crash_point_replay_feature_pass_cleared_environment：带特性那一趟 cargo 前面那一串 env -u（文件头「feature-gated-tests 格」：SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 与
#   此刻环境里每个 SINGLEFS_CRASH_AMPLIFICATION_*），放进全局数组 crash_point_replay_feature_cleared_environment；跑那一格与 --list-items --with-keys 算它的 ITEM_PLAN 都取这一份
crash_point_replay_feature_pass_cleared_environment() {
  local environment_name
  crash_point_replay_feature_cleared_environment=(-u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE)
  while IFS= read -r environment_name; do
    if [[ "$environment_name" == SINGLEFS_CRASH_AMPLIFICATION_* ]]; then crash_point_replay_feature_cleared_environment+=(-u "$environment_name"); fi
  done < <(compgen -e)
}
crash_point_replay_test_functions_request() {
  local request_output request_tag request_value
  local -a request_environment=()
  [[ "${GATE_CRASH_POINT_REPLAY_BREAK:-}" == test-functions-request-gate-environment ]] || request_environment=(${crash_point_replay_test_functions_environment[@]+"${crash_point_replay_test_functions_environment[@]}"})
  crash_point_replay_test_functions_kind=""; crash_point_replay_test_functions_arguments=()
  if request_output="$(env ${request_environment[@]+"${request_environment[@]}"} python3 "$2/scripts/gate/admission.py" test-functions-request "$1" "$2" 2>&1)"; then
    while IFS=$'\t' read -r request_tag request_value; do
      case "$request_tag" in
        kind) crash_point_replay_test_functions_kind="$request_value" ;;
        argument) crash_point_replay_test_functions_arguments+=("$request_value") ;;
      esac
    done <<<"$request_output"
  fi
  if [[ -z "$crash_point_replay_test_functions_kind" ]] || ((${#crash_point_replay_test_functions_arguments[@]} == 0)); then
    crash_point_replay_test_functions_kind="$1:test-functions-request-unreadable"
    crash_point_replay_test_functions_arguments=(--token "test-functions-request-unreadable:$(tr '\n\t' '  ' <<<"$request_output" | cut -c1-200)")
    return 1
  fi
  return 0
}
# crash_point_replay_plan_tokens <判法根> [<GPU 开关的值>]：每条测试函数的判定请求另加的项（文件头「复用」）放进全局数组 crash_point_replay_plan_token_arguments：
# 取 crash_point_replay_test_functions_request 交的（release 档、判法单元 lib_libtest_result_lines.py 的 recordable_results、判法根）。GPU 开没开不进键；
# 弄坏开关 gpu-switch-in-key 下照改前另加 line:enable-gpu=<第二个参数>
crash_point_replay_plan_tokens() {
  crash_point_replay_test_functions_request "$(basename "${BASH_SOURCE[0]}")" "$1"
  crash_point_replay_plan_token_arguments=("${crash_point_replay_test_functions_arguments[@]}")
  if [[ "${GATE_CRASH_POINT_REPLAY_BREAK:-}" == gpu-switch-in-key ]]; then crash_point_replay_plan_token_arguments+=(--token "line:enable-gpu=${2:-0}"); fi
  return 0
}
# crash_point_replay_verdict_kind <阶段文件名> <格名>：这一格按条复用记进、读的那一类判定库（文件头「复用」：几趟同一类、同一份库；种类取 crash_point_replay_test_functions_request 交的，
# 判法根取本阶段所在的仓）；弄坏开关 library-per-pass 下照改前一格一类（<阶段文件名>:<格名>）
crash_point_replay_verdict_kind() {
  if [[ "${GATE_CRASH_POINT_REPLAY_BREAK:-}" == library-per-pass ]]; then printf '%s:%s' "$1" "$2"; return 0; fi
  crash_point_replay_test_functions_request "$1" "${crash_point_replay_stage_repository:-${crash_point_replay_list_repository:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}}"
  printf '%s' "$crash_point_replay_test_functions_kind"
}
# list_feature_gated_test_files <tests 目录>：目录顶层每个 .rs（一个文件一个测试二进制）里 #[cfg(…)] / #![cfg(…)] 点到的特性，
# 一个文件一行「文件名<TAB>whole-file|items<TAB>特性,特性」（有 #![cfg(…)] 点到特性的记 whole-file），照文件名排序；挂特性的一个都没有时不打。
# 括号按层数配对（rustfmt 把长的 cfg 折成几行也认）；// 起头的行（注释与文档注释）不读。
# 写在参数解析之前：--list-items 列 feature-gated 的项（文件头「逐项点名」）也读它
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
# 逐项点名（文件头「逐项点名」）：feature-gated 的项的前缀、库那一项的名字
FEATURE_GATED_ITEM_PREFIX="feature-gated:"
FEATURE_GATED_LIBRARY_ITEM="feature-gated:lib"
crash_point_replay_named_feature_items=()   # --item 点名的 feature-gated 项（带前缀）
# crash_point_replay_name_item <项>：--item 的值按前缀分进两类（文件头「逐项点名」）：feature-gated: 起头的进 crash_point_replay_named_feature_items，别的当崩溃枚举用例
crash_point_replay_name_item() {
  if [[ "$1" == "$FEATURE_GATED_ITEM_PREFIX"* ]]; then crash_point_replay_named_feature_items+=("$1"); else crash_point_replay_named_cases+=("crash-case:${1#crash-case:}"); fi
}
# crash_point_replay_feature_items_of <tests 目录>：feature-gated 的可点的项，一项一行「项<TAB>说明」（库那一项在前；list_feature_gated_test_files 读不出返回 1）
crash_point_replay_feature_items_of() {
  local listing
  listing="$(list_feature_gated_test_files "$1")" || return 1
  [[ -n "$listing" ]] || return 0
  printf '%s\t--lib\n' "$FEATURE_GATED_LIBRARY_ITEM"
  awk -F'\t' -v prefix="$FEATURE_GATED_ITEM_PREFIX" '{ printf "%s%s\t%s %s\n", prefix, $1, $2, $3 }' <<<"$listing"
}
while (($#)); do
  stage_argument="$1"
  case "$stage_argument" in
    --list) crash_point_replay_list_cells=1 ;;
    --list-items) crash_point_replay_list_items=1 ;;
    --with-keys) crash_point_replay_list_with_keys=1 ;;
    --check|--item)
      if (($# < 2)) || [[ -z "$2" || "$2" == -* ]]; then
        echo "  ✗ $stage_argument 后面缺名字"
        echo "     → 怎么办：写成 --check $CRASH_CASE_REPLAY_CELL_NAME，或 --item <项>（崩溃枚举用例名，crash-case: 可写可不写；挂特性的测试文件写 feature-gated:<文件名>；几项就给几次；可点的用 --list-items 看）。"
        exit 2
      fi
      crash_point_replay_range_given=1
      if [[ "$stage_argument" == --check ]]; then crash_point_replay_check_cells "$2"; else crash_point_replay_name_item "$2"; fi
      shift ;;
    --check=*) crash_point_replay_range_given=1; crash_point_replay_check_cells "${stage_argument#--check=}" ;;
    --item=*) crash_point_replay_range_given=1; crash_point_replay_name_item "${stage_argument#--item=}" ;;
    -*)
      echo "  ✗ 认不出的参数：$stage_argument"
      echo "     → 怎么办：只认 --list、--check $CRASH_CASE_REPLAY_CELL_NAME、--list-items [--with-keys]、--item <项> 与项目根，顺序不限：bash .claude/gate.d/crash-point-replay.sh [--check <格名>[,<格名>…]] [--item <项>] [项目根]"
      exit 2 ;;
    *) root_argument="$stage_argument" ;;
  esac
  shift
done
if ((crash_point_replay_list_cells)); then
  crash_point_replay_print_cells
  exit 0
fi
if ((crash_point_replay_list_items)); then
  crash_point_replay_list_root="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}"
  crash_point_replay_list_table="$crash_point_replay_list_root/.claude/gate.d/stage-inputs.tsv"
  if [[ ! -f "$crash_point_replay_list_table" ]]; then
    echo "  ✗ 读不到 $crash_point_replay_list_table：列不出可点名的崩溃枚举用例"
    echo "     → 怎么办：在项目根跑，或把项目根作为参数传进来。"
    exit 1
  fi
  crash_point_replay_list_case_lines="$(awk -F'\t' '$1 ~ /^crash-case:/ { test = ""; n = split($3, tokens, " "); for (i = 1; i <= n; i++) if (tokens[i] ~ /^test=/) test = substr(tokens[i], 6); printf "%s\t%s\n", $1, test }' "$crash_point_replay_list_table")"
  [[ -z "$crash_point_replay_list_case_lines" ]] || printf '%s\n' "$crash_point_replay_list_case_lines"
  # feature-gated 的项（文件头「逐项点名」）：验证包 tests/ 里挂特性的测试文件现读
  crash_point_replay_list_tests_directory="$crash_point_replay_list_root/crates/singlefs-verification/tests"
  crash_point_replay_list_feature_lines=""
  if [[ -d "$crash_point_replay_list_tests_directory" ]] && ! crash_point_replay_list_feature_lines="$(crash_point_replay_feature_items_of "$crash_point_replay_list_tests_directory")"; then
    echo "  ✗ 读不出 $crash_point_replay_list_tests_directory 下哪几个测试文件挂特性 cfg（上面是 python 的报错）：列不出 feature-gated 的项"
    echo "     → 怎么办：看报错里点名的文件（多半不是 UTF-8 或读不了），修好再列。"
    exit 1
  fi
  [[ -z "$crash_point_replay_list_feature_lines" ]] || printf '%s\n' "$crash_point_replay_list_feature_lines"
  # 同时给了 --item 的：只给点名的项打 ITEM_PLAN；项写错退 2（同真跑）
  crash_point_replay_list_planned_items=()
  crash_point_replay_list_unknown_items=()
  for crash_point_replay_list_item in ${crash_point_replay_named_cases[@]+"${crash_point_replay_named_cases[@]}"} ${crash_point_replay_named_feature_items[@]+"${crash_point_replay_named_feature_items[@]}"}; do
    if cut -f1 <<<"$crash_point_replay_list_case_lines"$'\n'"$crash_point_replay_list_feature_lines" | grep -qxF -- "$crash_point_replay_list_item"; then
      crash_point_replay_list_planned_items+=("$crash_point_replay_list_item")
    else
      crash_point_replay_list_unknown_items+=("$crash_point_replay_list_item")
    fi
  done
  if ((${#crash_point_replay_list_unknown_items[@]})); then
    echo "  ✗ --item 点名的项不在可点的项里：${crash_point_replay_list_unknown_items[*]}"
    echo "     → 怎么办：项照上面列的第一列写（崩溃枚举用例 crash-case:<用例名>，挂特性的测试文件 feature-gated:<文件名>，库那一段 $FEATURE_GATED_LIBRARY_ITEM）。"
    exit 2
  fi
  if ((${#crash_point_replay_list_planned_items[@]} == 0)); then
    mapfile -t crash_point_replay_list_planned_items < <(cut -f1 <<<"$crash_point_replay_list_case_lines"$'\n'"$crash_point_replay_list_feature_lines" | grep .)
  fi
  # --with-keys：crash-case-replay 那一格有几条测试函数的键对不上（admission.py test-plan，只算键、不起 cargo），再每项一行 ITEM_PLAN（文件头「逐项点名」）
  if ((crash_point_replay_list_with_keys)); then
    crash_point_replay_list_repository="$(cd "$(dirname "$0")/../.." && pwd)"
    crash_point_replay_list_scratch="$(mktemp -d)"
    trap 'rm -rf -- "${crash_point_replay_list_scratch:?}"' EXIT
    if [[ -n "${GATE_CRASH_POINT_REPLAY_LIST_PLAN_OUTPUT:-}" ]]; then
      # 只供证红（文件头 --list-items）：照 test-plan 标准输出写法造的文件当这一趟的输出
      crash_point_replay_list_plan_command=(cat -- "$GATE_CRASH_POINT_REPLAY_LIST_PLAN_OUTPUT")
    else
      crash_point_replay_plan_tokens "$crash_point_replay_list_repository" 0
      crash_point_replay_list_plan_command=(python3 "$crash_point_replay_list_repository/scripts/gate/admission.py" test-plan "$crash_point_replay_list_root"
        "$(crash_point_replay_verdict_kind "$(basename "$0")" "$CRASH_CASE_REPLAY_CELL_NAME")" singlefs-verification
        "$crash_point_replay_list_scratch/plan" "${crash_point_replay_plan_token_arguments[@]}")
    fi
    if "${crash_point_replay_list_plan_command[@]}" >"$crash_point_replay_list_scratch/out" 2>&1 \
       && crash_point_replay_list_summary="$(grep '^PLAN ' "$crash_point_replay_list_scratch/out" | tail -n 1)" && [[ -n "$crash_point_replay_list_summary" ]]; then
      crash_point_replay_list_judge="$(sed -n 's/.* judge=\([0-9]*\).*/\1/p' <<<"$crash_point_replay_list_summary")"
      crash_point_replay_list_reuse="$(sed -n 's/.* reuse=\([0-9]*\).*/\1/p' <<<"$crash_point_replay_list_summary")"
      crash_point_replay_list_refused="$(sed -n 's/.* refused=\([0-9]*\).*/\1/p' <<<"$crash_point_replay_list_summary")"
      printf '%s\t键对不上的测试函数 %s 条（现判 %s、拒算 %s），键相同的 %s 条\n' "$CRASH_CASE_REPLAY_CELL_NAME" \
        "$((crash_point_replay_list_judge + crash_point_replay_list_refused))" "$crash_point_replay_list_judge" "$crash_point_replay_list_refused" "$crash_point_replay_list_reuse"
      # 拒算的逐条列：测试函数与理由（test-plan 一行的第一列与第七列，理由去掉「拒算：」起头），一条不落；弄坏开关 list-items-refusals-counted-only 下照改前不列
      if [[ ",${GATE_CRASH_POINT_REPLAY_BREAK:-}," != *,list-items-refusals-counted-only,* ]]; then
        awk -F'\t' -v cell="$CRASH_CASE_REPLAY_CELL_NAME" 'NF >= 7 && $2 == "refused" { reason = $7; sub(/^拒算：/, "", reason); printf "%s\t拒算\t%s\t%s\n", cell, $1, reason }' \
          "$crash_point_replay_list_scratch/out"
      fi
    else
      printf '%s\t键对不上的测试函数：算不出（%s）\n' "$CRASH_CASE_REPLAY_CELL_NAME" "$(tail -n 1 "$crash_point_replay_list_scratch/out")"
    fi
    # 每项一行 ITEM_PLAN（文件头「逐项点名」）：崩溃枚举用例一条一件、算要现判；feature-gated 的项照带特性那一趟的计划逐个测试目标数
    crash_point_replay_list_feature_plan_ok=0
    if [[ -n "$crash_point_replay_list_feature_lines" ]]; then
      crash_point_replay_list_feature_options=()
      while IFS= read -r crash_point_replay_list_feature; do
        [[ -n "$crash_point_replay_list_feature" ]] && crash_point_replay_list_feature_options+=(--feature "singlefs-verification/$crash_point_replay_list_feature")
      done < <(list_feature_gated_test_files "$crash_point_replay_list_tests_directory" | cut -f3 | tr ',' '\n' | LC_ALL=C sort -u)
      crash_point_replay_feature_pass_cleared_environment
      crash_point_replay_test_functions_environment=("${crash_point_replay_feature_cleared_environment[@]}")
      crash_point_replay_plan_tokens "$crash_point_replay_list_repository" 0
      if python3 "$crash_point_replay_list_repository/scripts/gate/admission.py" test-plan "$crash_point_replay_list_root" \
           "$(crash_point_replay_verdict_kind "$(basename "$0")" "$FEATURE_GATED_TESTS_CELL_NAME")" singlefs-verification "$crash_point_replay_list_scratch/features.plan" \
           "${crash_point_replay_plan_token_arguments[@]}" "${crash_point_replay_list_feature_options[@]}" --exclude-registered >"$crash_point_replay_list_scratch/features.out" 2>&1 \
         && grep -q '^PLAN ' "$crash_point_replay_list_scratch/features.out"; then
        crash_point_replay_list_feature_plan_ok=1
      else
        printf '%s\t按项的计划算不出，feature-gated 的项不打 ITEM_PLAN（%s）\n' "$FEATURE_GATED_TESTS_CELL_NAME" "$(tail -n 1 "$crash_point_replay_list_scratch/features.out")"
      fi
    fi
    for crash_point_replay_list_item in "${crash_point_replay_list_planned_items[@]}"; do
      if [[ "$crash_point_replay_list_item" == crash-case:* ]]; then
        printf 'ITEM_PLAN item=%s to_judge=1 reusable=0\n' "$crash_point_replay_list_item"
      elif ((crash_point_replay_list_feature_plan_ok)); then
        crash_point_replay_list_target_root="tests/${crash_point_replay_list_item#"$FEATURE_GATED_ITEM_PREFIX"}"
        [[ "$crash_point_replay_list_item" == "$FEATURE_GATED_LIBRARY_ITEM" ]] && crash_point_replay_list_target_root="src/lib.rs"
        awk -F'\t' -v item="$crash_point_replay_list_item" -v root="$crash_point_replay_list_target_root" '
          NF >= 7 && $5 == root { if ($2 == "judge" || $2 == "refused") judge++; else if ($2 == "reuse") reuse++ }
          END { printf "ITEM_PLAN item=%s to_judge=%d reusable=%d\n", item, judge, reuse }' "$crash_point_replay_list_scratch/features.out"
      fi
    done
    rm -rf -- "${crash_point_replay_list_scratch:?}"
  fi
  exit 0
fi
ROOT="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}"
# 相对的 $0 在 cd 之后会指错，在 cd 之前取成绝对路径
crash_point_replay_stage_script_path="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
# 准入模块：读登记表、算单元键、判前提、判日志、按条复用与崩溃状态块的判定都经它（子命令见它的文件头）
crash_point_replay_stage_repository="$(cd "$(dirname "$0")/../.." && pwd)"
crash_point_replay_admission_module="$crash_point_replay_stage_repository/scripts/gate/admission.py"
# 这一道的 cargo 直接起：编译与测试线程是服务给这一条套的 capped.sh 设好的 CARGO_BUILD_JOBS、RUST_TEST_THREADS 这些变量，这里不再套一层（capped.sh 只认服务套的那一层）。
cd "$ROOT" 2>/dev/null || exit 2
ROOT="$(pwd)"
stage_cells_refuse_unsampled_break_switches "$ROOT"
# 样本档（写法见文件头「样本档」）
crash_point_replay_sample_mode=0
if [[ -d "$PWD/.crash-point-replay-sample-tools" ]]; then
  crash_point_replay_sample_mode=1
  PATH="$PWD/.crash-point-replay-sample-tools:$PATH"
  export PATH
  # 样本档不接调用方环境里的 SINGLEFS_MULTI_HOST_CONFIG（经 task-server 交时它指那一台的真配置）：样本里的「调用方带来的多机配置」只从下面 caller-environment 来，
  #   判错的样本不随派到哪一台、经不经 task-server 变（文件头「多机配置开了 GPU」；弄坏开关 sample-caller-configuration-kept 只红在 caller-environment 写了它的那份样本上）
  unset SINGLEFS_MULTI_HOST_CONFIG
  # 样本档里 .crash-point-replay-sample-tools/caller-environment：一行一个「名字=值」，当作调用方的环境导出（样本借它模拟调用方设了改判定域的变量，文件头「样本档」）
  if [[ -f "$PWD/.crash-point-replay-sample-tools/caller-environment" ]]; then
    while IFS= read -r sample_environment_line; do
      [[ "$sample_environment_line" =~ ^[A-Z][A-Z0-9_]*= ]] && export "$sample_environment_line"
    done <"$PWD/.crash-point-replay-sample-tools/caller-environment"
  fi
  echo "  ! 样本档：$PWD/.crash-point-replay-sample-tools 在，起 cargo 的是那里面的假 cargo，按条复用照问样本自己的判定库；这一趟不算真跑过崩溃点重放"
fi
# 弄坏开关（只给证红用，清单在文件头）
crash_point_replay_break="${GATE_CRASH_POINT_REPLAY_BREAK:-}"
# crash_point_replay_apply_sample_marker：样本根的 .gate-cells 点名的格记进 crash_point_replay_selected_cells（文件头「样本只跑点名的格」）；
#   本阶段所在的仓（共用库的 STAGE_CELLS_STAGE_REPOSITORY）取 source 共用库时按本阶段的路径算的那一个
crash_point_replay_apply_sample_marker() {
  local marker_line marked_cells=()
  ((${#crash_point_replay_selected_cells[@]})) && return 0
  [[ -f .gate-cells ]] || return 0
  [[ "$crash_point_replay_break" == sample-marker-ignored ]] && return 0
  if [[ "$(pwd -P)" == "$STAGE_CELLS_STAGE_REPOSITORY" || -n "${GATE_STAGED_FROM:-}" ]]; then
    echo "  · $ROOT 里有 .gate-cells，但这里是本阶段所在的仓或 --staged 的临时树，不认它，照跑九格"
    return 0
  fi
  while IFS= read -r marker_line || [[ -n "$marker_line" ]]; do
    marker_line="${marker_line%%#*}"; marker_line="${marker_line//[[:space:]]/}"
    [[ -n "$marker_line" ]] && marked_cells+=("$marker_line")
  done < .gate-cells
  if ((${#marked_cells[@]} == 0)); then
    echo "  ✗ crash-point-replay.sh：样本根 $ROOT 的 .gate-cells 一个格名都没点"
    echo "     → 怎么办：一行写一个格名，可用的格名照下面写（也可以 bash .claude/gate.d/crash-point-replay.sh --list 看）："
    crash_point_replay_print_cells | sed 's/\t/  /; s/^/       /'
    exit 2
  fi
  crash_point_replay_check_cells "$(IFS=,; printf '%s' "${marked_cells[*]}")" "样本根 $ROOT 的 .gate-cells"
  echo "  · 样本只跑 .gate-cells 点名的格：${crash_point_replay_selected_cells[*]}（$ROOT 是样本，不是本阶段所在的仓）"
}
crash_point_replay_apply_sample_marker
# crash_point_replay_apply_sample_items：样本根的 .gate-items 当作命令行上的 --item（文件头「逐项点名」）：命令行没给 --item、被判的根是样本时才认
crash_point_replay_apply_sample_items() {
  local item_line marked_items=()
  (( ${#crash_point_replay_named_cases[@]} + ${#crash_point_replay_named_feature_items[@]} )) && return 0
  [[ -f .gate-items ]] || return 0
  if [[ "$(pwd -P)" == "$STAGE_CELLS_STAGE_REPOSITORY" || -n "${GATE_STAGED_FROM:-}" ]]; then
    echo "  · $ROOT 里有 .gate-items，但这里是本阶段所在的仓或 --staged 的临时树，不认它"
    return 0
  fi
  while IFS= read -r item_line || [[ -n "$item_line" ]]; do
    item_line="${item_line%%#*}"; item_line="${item_line//[[:space:]]/}"
    [[ -n "$item_line" ]] && marked_items+=("$item_line")
  done < .gate-items
  ((${#marked_items[@]})) || return 0
  for item_line in "${marked_items[@]}"; do crash_point_replay_name_item "$item_line"; done
  crash_point_replay_range_given=1
  echo "  · 样本点名 .gate-items 里的项：${marked_items[*]}（$ROOT 是样本，不是本阶段所在的仓）"
}
crash_point_replay_apply_sample_items
# feature-gated 的项（文件头「逐项点名」）：认一遍，写错退 2；没给 --check（也没有 .gate-cells）而没点崩溃枚举用例的只跑 feature-gated-tests 一格，
#   点的格里没有 feature-gated-tests 退 2。弄坏开关 feature-item-unknown-accepted 下写错的悄悄丢掉
crash_point_replay_feature_items_listing=""
if ((${#crash_point_replay_named_feature_items[@]})); then
  if ! crash_point_replay_feature_items_listing="$(crash_point_replay_feature_items_of crates/singlefs-verification/tests 2>&1)"; then
    printf '%s\n' "$crash_point_replay_feature_items_listing" | sed 's/^/       /'
    echo "  ✗ 读不出 crates/singlefs-verification/tests 下哪几个测试文件挂特性 cfg（上面是 python 的报错）：认不了 --item 点名的 feature-gated 项"
    echo "     → 怎么办：看报错里点名的文件（多半不是 UTF-8 或读不了），修好再跑。"
    exit 1
  fi
  crash_point_replay_feature_items_kept=()
  crash_point_replay_feature_items_unknown=()
  for crash_point_replay_feature_item in "${crash_point_replay_named_feature_items[@]}"; do
    if cut -f1 <<<"$crash_point_replay_feature_items_listing" | grep -qxF -- "$crash_point_replay_feature_item"; then
      crash_point_replay_cell_in "$crash_point_replay_feature_item" ${crash_point_replay_feature_items_kept[@]+"${crash_point_replay_feature_items_kept[@]}"} \
        || crash_point_replay_feature_items_kept+=("$crash_point_replay_feature_item")
    else
      crash_point_replay_feature_items_unknown+=("$crash_point_replay_feature_item")
    fi
  done
  if ((${#crash_point_replay_feature_items_unknown[@]})) && [[ "$crash_point_replay_break" != feature-item-unknown-accepted ]]; then
    echo "  ✗ --item 点名的 feature-gated 项不在可点的项里：${crash_point_replay_feature_items_unknown[*]}"
    echo "     → 怎么办：照下面可点的项写（也可以 bash .claude/gate.d/crash-point-replay.sh --list-items 看）："
    sed 's/\t/  /; s/^/       /' <<<"${crash_point_replay_feature_items_listing:-（crates/singlefs-verification/tests 下没有挂特性的测试文件，一项都没有）}"
    exit 2
  fi
  crash_point_replay_named_feature_items=(${crash_point_replay_feature_items_kept[@]+"${crash_point_replay_feature_items_kept[@]}"})
  if ((${#crash_point_replay_selected_cells[@]} == 0)) && ((${#crash_point_replay_named_cases[@]} == 0)); then
    crash_point_replay_check_cells "$FEATURE_GATED_TESTS_CELL_NAME" "--item 点名的 feature-gated 项"
    echo "  · 只点了 feature-gated 的项、没给 --check：只跑 $FEATURE_GATED_TESTS_CELL_NAME 一格"
  elif ! crash_point_replay_runs_cell "$FEATURE_GATED_TESTS_CELL_NAME"; then
    echo "  ✗ --item 点名的 feature-gated 项（${crash_point_replay_named_feature_items[*]}）不在这一趟点的格里：点的格是 ${crash_point_replay_selected_cells[*]}"
    echo "     → 怎么办：--check 里加上 $FEATURE_GATED_TESTS_CELL_NAME，或不给 --check（只点 feature-gated 的项时只跑那一格）。"
    exit 2
  fi
fi
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
    echo "                缺的照 .claude/gate.d/fixtures/crash-point-replay.sh/green/.crash-point-replay-sample-tools 补上，再跑；不是有意跑样本档的，把 $crash_point_replay_sample_tools_directory 挪走再跑。"
    exit 2
  fi
fi

# librocksdb-sys 的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：起任何 cargo 之前判一次这台装了、此刻进程环境的取值对得上，不过就退 1、一个 cargo 都不起
#（不然 验证包经它依赖的 prune-server 在编译目录里现编 RocksDB 的 C++）。树里没有依赖 rocksdb 的清单（样本多是这样）、PATH 上的 cargo 是带标记的假 cargo，entry 自己报不判、退 0。
# 判法脚本不在（admission.py --selftest 拷这一份去的临时仓没有 scripts/hosts/）报一行本次未跑、往下走。弄坏开关 rocksdb-entry-unchecked 拿掉这一段（样本 rocksdb-entry-refused 判错）
crash_point_replay_rocksdb_entry="$crash_point_replay_stage_repository/scripts/hosts/rocksdb-prebuilt.py"
if [[ "$crash_point_replay_break" == rocksdb-entry-unchecked ]]; then
  :
elif [[ ! -f "$crash_point_replay_rocksdb_entry" ]]; then
  echo "  ! RocksDB 预编库的取值没判：$crash_point_replay_rocksdb_entry 不在（这一份 crash-point-replay 被拷到了不带 scripts/hosts/ 的树里），往 GATE_NOT_RUN_FILE 报一行"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
    printf '%s\n' "crash-point-replay.sh 起 cargo 之前没判 RocksDB 预编库的取值：$crash_point_replay_rocksdb_entry 不在" >> "$GATE_NOT_RUN_FILE"
  fi
elif ! python3 "$crash_point_replay_rocksdb_entry" entry --root "$ROOT"; then
  exit 1
fi

# 开跑前的静态判据（placement、modules；判法在 scripts/crash/crash-case-check.py）：真跑与样本档都在复用判定之前判，红了不起 cargo。
# 判法脚本不在（admission.py --selftest 拷这一份去的临时仓没有它）报一行本次未跑、往下走，不判绿也不判红。
crash_point_replay_static_judge="$crash_point_replay_stage_repository/scripts/crash/crash-case-check.py"
if [[ -d crates && "$crash_point_replay_break" != static-check-skipped && ! -f "$crash_point_replay_static_judge" ]]; then
  echo "  ! 开跑前的静态判据没跑：判法脚本 $crash_point_replay_static_judge 不在（这一份 crash-point-replay 被拷到了不带 scripts/ 的树里），往 GATE_NOT_RUN_FILE 报一行"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
    printf '%s\n' "crash-point-replay.sh 开跑前的静态判据（placement、modules）没跑：$crash_point_replay_static_judge 不在" >> "$GATE_NOT_RUN_FILE"
  fi
elif [[ -d crates && "$crash_point_replay_break" != static-check-skipped ]]; then
  echo "── 开跑前的静态判据：崩溃枚举用例住哪、登记没有，验证包测试文件声明模块"
  if python3 "$crash_point_replay_static_judge" --only placement,modules "$ROOT"; then crash_point_replay_static_exit=0; else crash_point_replay_static_exit=$?; fi
  if [[ "$crash_point_replay_static_exit" != 0 && "$crash_point_replay_static_exit" != 77 ]]; then
    echo "  ✗ 开跑前的静态判据：crash-case-check --only placement,modules 退 $crash_point_replay_static_exit（逐处列在上面），这一趟不起 cargo"
    echo "     → 怎么办：照上面那一句出路改（崩溃枚举用例挪进 crates/singlefs-verification/tests/、每条都登记进 .claude/gate.d/stage-inputs.tsv 的 crash-case: 行；"
    echo "                验证包测试文件第一行写「//! 验证包模块：…」）；改完先单跑 python3 scripts/crash/crash-case-check.py --only placement,modules 到它退 0，再跑这一道。"
    exit 1
  fi
fi

crash_point_replay_stage_file_name="$(basename "$0")"
[[ -f Cargo.toml && -d crates/singlefs-verification ]] || { echo "  ! 没有 crates/singlefs-verification（验证包），本阶段跳过"; exit 77; }
python3 "$crash_point_replay_admission_module" gate-preconditions "$crash_point_replay_stage_repository" "$crash_point_replay_stage_file_name" || exit 1

if ! git_common_directory="$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$git_common_directory" ]]; then
  echo "  ✗ $ROOT 不是 git 工作树（取不到 git common-dir）：每条用例的日志没处放，单元键要 git 列文件也算不出"
  echo "     → 怎么办：在这个项目的 git 仓里跑，或把仓库根作为参数传进来；判定在 prune-server 那一份库里，不在 git common-dir 下。"
  exit 1
fi
crash_point_replay_scratch_directory="$(mktemp -d)"
# 判红时留下的日志：一份一行「<路径><TAB><留下那一刻的 sha256>」，收尾的 crash_point_replay_exit_trap 读
crash_point_replay_kept_logs=()
# 每一趟 cargo 的整份输出放的目录（文件头「每一趟当场写结局」）：在草稿目录之外、${TMPDIR:-/tmp} 底下，下面建；
# crash_point_replay_pass_logs_disposable 在这一道判绿收尾之前置 1，EXIT trap 照它删或留
crash_point_replay_pass_log_directory=""
crash_point_replay_pass_logs_disposable=0
# crash_point_replay_exit_trap：这一道的 EXIT trap：删掉草稿目录；每一趟 cargo 的整份输出判绿收尾（或样本档、或一份都没写）时删，
# 别的（判红、被 TERM 打断，被杀时 bash 也走这个 trap，那时 $? 读不准，只认判绿收尾前置的那个变量）留着、打一行路径；
# 样本档里再逐份核判红时留下的日志在草稿目录删掉之后还在、内容与留下那一刻逐字节相同，核完删掉（文件头「样本档」）；真仓上留着给人看。不改这一道的退出码
crash_point_replay_exit_trap() {
  rm -rf -- "${crash_point_replay_scratch_directory:?}"
  if [[ -n "$crash_point_replay_pass_log_directory" && -d "$crash_point_replay_pass_log_directory" ]]; then
    if [[ "$crash_point_replay_sample_mode" == 1 || "$crash_point_replay_pass_logs_disposable" == 1 || -z "$(ls -A -- "$crash_point_replay_pass_log_directory")" ]]; then
      rm -rf -- "${crash_point_replay_pass_log_directory:?}"
    else
      echo "  · 每一趟 cargo 的整份输出留在 ${crash_point_replay_pass_log_directory}/（这一道没判绿收尾：判红、被打断或被杀；看完自己删）"
    fi
  fi
  [[ "$crash_point_replay_sample_mode" == 1 ]] || return 0
  local kept_entry kept_path kept_digest
  for kept_entry in ${crash_point_replay_kept_logs[@]+"${crash_point_replay_kept_logs[@]}"}; do
    kept_path="${kept_entry%%$'\t'*}"
    kept_digest="${kept_entry#*$'\t'}"
    if [[ -f "$kept_path" && "$(sha256sum < "$kept_path" | cut -d' ' -f1)" == "$kept_digest" ]]; then
      echo "  · 样本档：判红时留下的 $kept_path 在这一道的草稿目录删掉之后还在、内容与留下那一刻逐字节相同（sha256 ${kept_digest:0:12}）；核完删掉"
      rm -f -- "$kept_path"
    else
      echo "  ✗ 样本档：判红时留下的 $kept_path 在这一道的草稿目录删掉之后不在了（或内容变了）：出路指的那一份跑完就没了"
      echo "     → 怎么办：判红时把日志拷到 \${TMPDIR:-/tmp} 下（mktemp），出路指拷出来的那一份，不指这一道草稿目录里的"
    fi
  done
}
trap crash_point_replay_exit_trap EXIT
if ! crash_point_replay_pass_log_directory="$(mktemp -d "${TMPDIR:-/tmp}/crash-point-replay-passes.XXXXXX")"; then
  crash_point_replay_pass_log_directory=""
  echo "  ✗ 建不了放每一趟 cargo 整份输出的目录（mktemp -d 在 ${TMPDIR:-/tmp} 下失败），一趟 cargo 都没起"
  echo "     → 怎么办：看 ${TMPDIR:-/tmp} 在不在、可不可写、盘满没满（df -h ${TMPDIR:-/tmp}），修好之后重跑这一道"
  exit 1
fi
# 按条复用两趟的整份输出（文件头「每一趟当场写结局」）：不带特性那一趟、带特性那一趟（放开 --test-threads 1 的另起一趟在它旁边，名字加 .released；
# 并成一份之前带 --test-threads 1 的那一趟在 .main）
crash_point_replay_plain_pass_log="$crash_point_replay_pass_log_directory/plain-singlefs-verification.log"
crash_point_replay_feature_pass_log="$crash_point_replay_pass_log_directory/features-singlefs-verification.log"
# 弄坏开关 pass-outcome-at-cell-end 下每一趟的结局先记在这里，整格收尾才打（report_pass_outcome、flush_deferred_pass_outcomes）
crash_point_replay_deferred_pass_outcomes="$crash_point_replay_scratch_directory/deferred-pass-outcomes"
# 不带特性那一趟打了 NOT_RUN_ 的测试函数（not_run_markers_of 的写法）与带特性那一趟补判的结论（文件头「多机配置开了 GPU」）
crash_point_replay_plain_not_run_file="$crash_point_replay_scratch_directory/plain-not-run"
crash_point_replay_supplement_file="$crash_point_replay_scratch_directory/plain-not-run.supplement"
crash_point_replay_plain_not_run_total=0
: >"$crash_point_replay_plain_not_run_file"

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
  echo "  ✗ .claude/gate.d/stage-inputs.tsv 里一条崩溃枚举用例（键是 crash-case: 的行）都没登记：崩溃点重放没有东西可跑、可核"
  echo "     → 怎么办：两条流的崩溃枚举用例至少各登记一行，写法见 scripts/gate/admission.py 文件头「崩溃枚举用例行」。"
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
    echo "     → 怎么办：用例名照 bash .claude/gate.d/crash-point-replay.sh --list-items 列的写（登记表里有 ${#crash_case_rows[@]} 条）。"
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


# ── 按条复用（文件头「复用」）：每条测试函数一条判定 ─────────
# plan_tests <种类> <计划文件> <计划输出> [--feature <包>/<特性>…]：admission.py test-plan 算 验证包（局部变量 plan_package 设了就算那个包：product-gpu-real-card-tests 格设成产品 GPU 包）里每条测试函数的键、查库，输出一条一行
#   （<名字>、<reuse|judge|refused|not-compiled>、<键>、<目标号>、<目标根文件>、<libtest 全名>、<原因>）落进 <计划输出>，末行 PLAN 那一句的数放进
#   plan_judge、plan_reuse、plan_refused、plan_not_compiled；算不出（工作区读不出、模块出错）返回 1、原因放进 plan_problem、那一趟的子命令与退出码与输出放进 plan_failure，
#   调用方全部现判、什么都不记，再经 plan_failure_verdict 判按条复用失灵（失灵那一格结论判红）。
#   样本档放了 .unit-verdict-sample/prior/ 的，先拿上一版的树算一遍、照判绿记进这一版的判定库（文件头「样本档」）。
#   弄坏开关 GATE_CRASH_POINT_REPLAY_BREAK=setting-file-whole-closure 下每条测试函数的键另加它所在的整份设点文件（admission.py 的 ADMISSION_BREAK=test-key-with-whole-file）。
plan_tests() {
  local kind="$1" plan_file="$2" plan_output="$3" summary plan_exit
  shift 3
  local -a feature_options=("$@") break_setting=() registered_exclusion=()
  # 登记的崩溃枚举用例与放量分项（不标 #[ignore]，交按块那一路）不进按条复用的计划：不当复用哨兵、不被强制现判、样本档不替它记绿；
  # 只给整包那几趟（plan_package 空着），产品 GPU 包那一格没有登记的用例。弄坏开关 registered-rows-in-plan 下照改前不带
  [[ -z "${plan_package:-}" && "$crash_point_replay_break" != registered-rows-in-plan ]] && registered_exclusion=(--exclude-registered)
  crash_point_replay_plan_tokens "$crash_point_replay_stage_repository" "$crash_point_replay_gpu_configured"
  local -a tokens=("${crash_point_replay_plan_token_arguments[@]}")
  [[ "$crash_point_replay_break" == setting-file-whole-closure ]] && break_setting=("ADMISSION_BREAK=test-key-with-whole-file")
  plan_judge=0; plan_reuse=0; plan_refused=0; plan_not_compiled=0; plan_problem=""; plan_failure=()
  if [[ "$crash_point_replay_sample_mode" == 1 && -d "$PWD/.unit-verdict-sample/prior" ]]; then
    seed_prior_verdicts "$kind" ${feature_options[@]+"${feature_options[@]}"} || return 1
  fi
  if env ${break_setting[@]+"${break_setting[@]}"} python3 "$crash_point_replay_admission_module" test-plan "$ROOT" "$kind" "${plan_package:-singlefs-verification}" "$plan_file" \
       "${tokens[@]}" ${feature_options[@]+"${feature_options[@]}"} ${registered_exclusion[@]+"${registered_exclusion[@]}"} >"$plan_output" 2>"$plan_output.err"; then plan_exit=0; else plan_exit=$?; fi
  if ((plan_exit != 0)); then
    plan_problem="admission.py test-plan 没成：$(tail -n 3 "$plan_output" "$plan_output.err" 2>/dev/null | tr '\n' ' ' | cut -c1-400)"
    plan_failure=(test-plan "$plan_exit" "$plan_output" "$plan_output.err")
    return 1
  fi
  [[ -z "${plan_package:-}" ]] && drop_registered_rows_from_plan "$plan_output"
  summary="$(grep '^PLAN ' "$plan_output" | tail -n 1)"
  [[ -n "$summary" ]] || { plan_problem="admission.py test-plan 没打 PLAN 那一行"; plan_failure=(test-plan 0 "$plan_output" "$plan_output.err"); return 1; }
  plan_judge="$(sed -n 's/.* judge=\([0-9]*\).*/\1/p' <<<"$summary")"
  plan_reuse="$(sed -n 's/.* reuse=\([0-9]*\).*/\1/p' <<<"$summary")"
  plan_refused="$(sed -n 's/.* refused=\([0-9]*\).*/\1/p' <<<"$summary")"
  plan_not_compiled="$(sed -n 's/.* not-compiled=\([0-9]*\).*/\1/p' <<<"$summary")"
  return 0
}

# seed_prior_verdicts <种类> [--feature …]：样本档（文件头「样本档」）：拿 .unit-verdict-sample/prior/ 那一版算每条测试函数的键、全部照判绿记进这一版样本的判定库。
# 样本另放了 .unit-verdict-sample/prior-multi-host.env（上一版那一趟的多机配置）的，这一趟照它起（SINGLEFS_MULTI_HOST_CONFIG），GPU 开关取它的 ENABLE_GPU
# （只在弄坏开关 gpu-switch-in-key 下进键）
seed_prior_verdicts() {
  local kind="$1" prior="$PWD/.unit-verdict-sample/prior" seed="$crash_point_replay_scratch_directory/seed" seed_exit
  local prior_configuration="$PWD/.unit-verdict-sample/prior-multi-host.env" prior_switch="$crash_point_replay_gpu_configured" prior_note=""
  shift
  local -a break_setting=() prior_setting=() registered_exclusion=()
  [[ "$crash_point_replay_break" == setting-file-whole-closure ]] && break_setting=("ADMISSION_BREAK=test-key-with-whole-file")
  [[ -z "${plan_package:-}" && "$crash_point_replay_break" != registered-rows-in-plan ]] && registered_exclusion=(--exclude-registered)
  if [[ -f "$prior_configuration" ]]; then
    prior_setting=("SINGLEFS_MULTI_HOST_CONFIG=$prior_configuration")
    prior_switch=0
    if [[ "$(source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_configuration_value "$prior_configuration" ENABLE_GPU)" == 1 ]]; then prior_switch=1; fi
    prior_note="（上一版那一趟的多机配置 .unit-verdict-sample/prior-multi-host.env，ENABLE_GPU=${prior_switch}）"
  fi
  crash_point_replay_plan_tokens "$crash_point_replay_stage_repository" "$prior_switch"
  if env ${prior_setting[@]+"${prior_setting[@]}"} ${break_setting[@]+"${break_setting[@]}"} python3 "$crash_point_replay_admission_module" test-plan "$prior" "$kind" "${plan_package:-singlefs-verification}" "$seed.plan" \
       "${crash_point_replay_plan_token_arguments[@]}" "$@" ${registered_exclusion[@]+"${registered_exclusion[@]}"} >"$seed.out" 2>&1; then seed_exit=0; else seed_exit=$?; fi
  if ((seed_exit != 0)); then
    plan_problem="样本档：上一版的树上算不出键：$(tail -n 3 "$seed.out" | tr '\n' ' ' | cut -c1-400)"
    plan_failure=(test-plan "$seed_exit" "$seed.out")
    return 1
  fi
  awk -F'\t' '$2 == "judge" || $2 == "reuse" { print $1 }' "$seed.out" >"$seed.names"
  if python3 "$crash_point_replay_admission_module" verdict-record "$prior" "$kind" "$seed.plan" "$seed.names" --how "样本档：上一版" >"$seed.record" 2>&1; then
    seed_exit=0; else seed_exit=$?; fi
  if ((seed_exit != 0)); then
    plan_problem="样本档：上一版那一趟的判定没记进库：$(tail -n 2 "$seed.record" | tr '\n' ' ')"; plan_failure=(verdict-record "$seed_exit" "$seed.record"); return 1; fi
  echo "  · 样本档：上一版（.unit-verdict-sample/prior）的 $(wc -l <"$seed.names") 条测试函数照判绿记进这一版的判定库（$(tail -n 1 "$seed.record")）${prior_note}"
}

# skip_arguments_of_plan <计划输出|->：登记的崩溃枚举用例与放量分项（crash_point_replay_registered_pairs_file 的用例函数，只在 验证包那两趟）逐条 --skip，
# 再是键相同（reuse）、这一组特性下不编（not-compiled）的测试函数逐条 --skip（加 --exact）；要现判的同名就不跳（几个二进制里同名的测试都跑，宁多跑）。
# 计划里的名单只在 scripts/gate/admission.py 的 plan-skip-names 一处（它文件头「按条复用的共用段」，scripts/crash/crash-point-replay.sh 按项跑的那一段调同一条）；
# 给 - 是计划算不出，只排登记的。结果放进全局数组 plan_skip_arguments（cargo test 的 -- 之后接）。取不出计划里的名单就只排登记的（其余全部现判，宁多跑）、打一行。
# 弄坏开关 registered-cases-not-skipped 下不排登记的
skip_arguments_of_plan() {
  local name names="" registered_names=""
  plan_skip_arguments=()
  if [[ "$1" != - ]] && ! names="$(python3 "$crash_point_replay_admission_module" plan-skip-names "$1")"; then
    echo "  ! 按条复用：admission.py plan-skip-names 没成（$(tr '\n' ' ' <<<"$names" | cut -c1-300)），这一趟计划里的一条都不跳、全部现判"
    names=""
  fi
  if [[ -z "${plan_package:-}" && "$crash_point_replay_break" != registered-cases-not-skipped && -s "$crash_point_replay_registered_pairs_file" ]]; then
    registered_names="$(cut -f2 "$crash_point_replay_registered_pairs_file" | LC_ALL=C sort -u)"
  fi
  while IFS= read -r name; do
    [[ -n "$name" ]] && plan_skip_arguments+=(--skip "$name")
  done <<<"$registered_names"$'\n'"$names"
  ((${#plan_skip_arguments[@]})) && plan_skip_arguments=(--exact "${plan_skip_arguments[@]}")
  return 0
}

# plan_failure_verdict <格名> <子命令> <退出码> <输出文件>…：按条复用失灵没有（判法只在 scripts/gate/admission.py 的 plan-failure，文件头「按条复用失灵」）；
# 失灵它打 ✗ 与出路、这里返回 1，那一格照旧跑完、结论判红。它退 75 是 prune-server 不可用（不是失灵）：这一格记进 crash_point_replay_unavailable_cells、
# 这里返回 75，调用方让这一格就此收尾（计划那一步退 75 的不起 cargo），别的格照跑；整道收尾时有别的红退 1、两样都报，只有不可用退 75（文件头「退出码」）。
# 弄坏开关 plan-unavailable-as-failure 下照失灵办（返回 1）；plan-unavailable-exits-at-once 下照改前当场整道退 75（前面已判红的格被吞）
plan_failure_verdict() {
  local cell="$1" verdict_exit
  shift
  if python3 "$crash_point_replay_admission_module" plan-failure "$crash_point_replay_stage_file_name 的 $cell 格" "$@"; then return 0; else verdict_exit=$?; fi
  if (( verdict_exit == 75 )) && [[ "$crash_point_replay_break" != plan-unavailable-as-failure ]]; then
    if [[ "$crash_point_replay_break" == plan-unavailable-exits-at-once ]]; then
      echo "  ✗ $cell 这一格没判：prune-server 不可用（上面那一句）；这一道不往下跑，退 75（不是红，也不算通过）"
      echo "     → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；修好再重跑这一道"
      exit 75
    fi
    echo "  ✗ $cell 这一格没判：prune-server 不可用（上面那一句）；这一格就此收尾，别的格照跑（不是红，也不算通过）"
    echo "     → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；修好只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $cell [项目根]"
    crash_point_replay_cell_in "$cell" ${crash_point_replay_unavailable_cells[@]+"${crash_point_replay_unavailable_cells[@]}"} \
      || crash_point_replay_unavailable_cells+=("$cell")
    return 75
  fi
  return 1
}

# crash_point_replay_unavailable_only <格名>：这一格判不了只因为 prune-server 不可用（在 crash_point_replay_unavailable_cells 里、不在 crash_point_replay_red_regardless_cells 里）退 0
crash_point_replay_unavailable_only() {
  crash_point_replay_cell_in "$1" ${crash_point_replay_unavailable_cells[@]+"${crash_point_replay_unavailable_cells[@]}"} \
    && ! crash_point_replay_cell_in "$1" ${crash_point_replay_red_regardless_cells[@]+"${crash_point_replay_red_regardless_cells[@]}"}
}

# red_cell_state <格名>：逐格汇总里没过的那一格写什么：只因 prune-server 不可用的写「prune-server 不可用」，别的写「红」
red_cell_state() {
  if crash_point_replay_unavailable_only "$1"; then printf 'prune-server 不可用'; else printf '红'; fi
}

# rerun_check_list <格名>…：只重跑这几格的 --check 列表（逗号隔开）。覆盖、ignore、0 次三格读按条复用那两趟与每条用例的行：点了其中一格而没点 crash-case-replay 的，
#   连 crash-case-replay 与 feature-gated-tests 一起点
rerun_check_list() {
  local rerun_cells=("$@") flow_cell
  for flow_cell in "${CRASH_AUDIT_FLOW_CELLS[@]}"; do
    if crash_point_replay_cell_in "$flow_cell" "$@" && ! crash_point_replay_cell_in "$CRASH_CASE_REPLAY_CELL_NAME" "${rerun_cells[@]}"; then
      crash_point_replay_cell_in "$FEATURE_GATED_TESTS_CELL_NAME" "${rerun_cells[@]}" || rerun_cells=("$FEATURE_GATED_TESTS_CELL_NAME" "${rerun_cells[@]}")
      rerun_cells=("$CRASH_CASE_REPLAY_CELL_NAME" "${rerun_cells[@]}")
    fi
  done
  (IFS=,; printf '%s' "${rerun_cells[*]}")
}

# account_test_results <日志> <这一趟带没带 --test-threads 1：1|0> <清单文件>：逐条认结局与记号归谁（判法只在 scripts/gate/lib_libtest_result_lines.py 的
# recordable_results，文件头「按条记库」；这里只调它的 recordable 子命令），一行一条落进 <清单文件>（green / withheld / marker，写法在它文件头「命令行」）。
# 没成返回 1、打一行原因（这一趟一条都不记，本次未跑照 test_not_run_lines 报）。弄坏开关 withhold-whole-binary 下转成它的 marker-withholds-whole-binary 与
# count-mismatch-withholds-whole-binary（改前一段有记号、或结果行条数与汇总行对不上就整段不记）。
# 多机配置没开 GPU（crash_point_replay_gpu_configured 是 0）时带 --cpu-judging-recordable：判段判在 CPU 的那一条判绿照键记库（文件头「复用」）；
# 开了 GPU 不带（判段判在 CPU 的那一趟判红，判归哪一条那一条不记绿；归得到的另记红、归不到的整个二进制不记，record_cpu_judging_pass）。弄坏开关 cpu-judging-withheld 下没开 GPU 也不带（改前一律不记），
# cpu-judging-recorded-when-gpu-configured 下开了 GPU 也带
account_test_results() {
  local -a break_setting=() serial_option=() cpu_judging_option=()
  [[ "$crash_point_replay_break" == withhold-whole-binary ]] && break_setting=("LIBTEST_RESULT_LINES_BREAK=marker-withholds-whole-binary,count-mismatch-withholds-whole-binary")
  [[ "$2" == 1 ]] && serial_option=(--serial)
  if { (( ! crash_point_replay_gpu_configured )) && [[ "$crash_point_replay_break" != cpu-judging-withheld ]]; } \
       || [[ "$crash_point_replay_break" == cpu-judging-recorded-when-gpu-configured ]]; then
    cpu_judging_option=(--cpu-judging-recordable)
  fi
  if env ${break_setting[@]+"${break_setting[@]}"} python3 "$crash_point_replay_stage_repository/scripts/gate/lib_libtest_result_lines.py" recordable "$1" \
       ${serial_option[@]+"${serial_option[@]}"} ${cpu_judging_option[@]+"${cpu_judging_option[@]}"} >"$3" 2>"$3.err" && grep -q '^RECORDABLE ' "$3"; then
    return 0
  fi
  echo "  ! 按条记库：lib_libtest_result_lines.py recordable 没成（$(tail -n 2 "$3" "$3.err" 2>/dev/null | tr '\n' ' ' | cut -c1-300)），这一趟一条都不记"
  return 1
}

# account_feature_pass <带特性那一趟的日志> <那一趟带没带 --test-threads 1：1|0> <放开 --test-threads 1 另起的那一趟的日志，没另起写 -> <清单文件>：
# feature-gated-tests 格带特性那一趟逐条认结局（account_test_results）；另起了放开的那一趟的，那一趟另认（不带 --serial），认出的行并进同一份清单
# （判绿记库与补判都按行读）。哪一趟认不成返回 1（这一趟一条都不记；带特性那一趟已认出的行留在清单里，补判照读）
account_feature_pass() {
  account_test_results "$1" "$2" "$4" || return 1
  [[ "$3" != - ]] || return 0
  account_test_results "$3" 0 "$4.released" || return 1
  cat -- "$4.released" >> "$4"
}

# green_names_of_pass <计划输出> <清单文件> <名字文件> <行头>：这一趟现判而判绿、可记库的测试函数名字落进 <名字文件>（<清单文件> 是 account_test_results 认出的，
# 只取 green 那几条），现判而不记的逐条列出为什么（至多 20 条）。取不出名单打一行「! <行头>：…」、<名字文件> 清空、返回 1（这一趟一条都不记）。
# 判绿的名单只在 scripts/gate/admission.py 的 plan-green-names 一处（它文件头「按条复用的共用段」，scripts/crash/crash-point-replay.sh 按项跑的那一段调同一条）
green_names_of_pass() {
  local plan_output="$1" accounting="$2" names="$3" label="$4" withheld_count
  if ! python3 "$crash_point_replay_admission_module" plan-green-names "$plan_output" "$accounting" "$names.withheld" >"$names"; then
    echo "  ! ${label}：取不出判绿的名单（admission.py plan-green-names：$(tr '\n' ' ' <"$names" | cut -c1-300)），这一趟一条都不记"
    : >"$names"
    return 1
  fi
  withheld_count="$(grep -c . "$names.withheld" 2>/dev/null)"
  if (( ${withheld_count:-0} > 0 )); then
    echo "  · 判绿不记库 ${withheld_count} 条（这一趟现判、按条记库不认的；下一趟照跑）："
    head -n 20 "$names.withheld" | awk -F'\t' '{ printf "    不记 %s（%s）\n", $1, substr($2, 1, 300) }'
    (( withheld_count > 20 )) && echo "    …另有 $((withheld_count - 20)) 条没列"
  fi
  return 0
}

# record_green_tests <种类> <计划文件> <计划输出> <清单文件> <格名> <跑法>：这一趟现判而判绿的测试函数照键记进库（文件头「复用」：名单经 green_names_of_pass 取）。
# 打一行「· 判绿记库：…」；记不下只报、不判红；复用哨兵报红（强制现判那一条判定不稳）的经 plan_failure_verdict 判失灵、返回 1。
# 那一趟 cargo test 退非 0 的不走这里，走 record_red_tests（判红的与判绿的同一次记）
record_green_tests() {
  local kind="$1" plan_file="$2" plan_output="$3" accounting="$4" cell="$5" how="$6" names="$crash_point_replay_scratch_directory/green-names.$5" record_exit
  green_names_of_pass "$plan_output" "$accounting" "$names" 判绿记库 || return 0
  if [[ ! -s "$names" ]]; then echo "  · 判绿记库：这一趟没有可记的（现判的都不认，或一条都没现判）"; return 0; fi
  if python3 "$crash_point_replay_admission_module" verdict-record "$ROOT" "$kind" "$plan_file" "$names" --how "$how" \
       >"$names.record" 2>&1; then record_exit=0; else record_exit=$?; fi
  if ((record_exit == 0)); then
    echo "  · 判绿记库：$(tail -n 1 "$names.record")"
  else
    echo "  ! 判绿记库：admission.py verdict-record 没成（$(tail -n 2 "$names.record" | tr '\n' ' ')），下一趟这几条照跑"
    plan_failure_verdict "$cell" verdict-record "$record_exit" "$names.record" || return 1
  fi
  return 0
}

# record_red_tests <种类> <计划文件> <计划输出> <红的测试函数清单> <格名> <跑法> [<清单文件>]：按条复用那一趟判红时（文件头「复用」记红、撤绿、下趟现判），
#   report_red_test_targets 落下的「<源文件><TAB><libtest 全名>」按（目标根文件，libtest 全名）对到计划里 judge、reuse、refused 的那一行，取它第一列的名字，
#   经 admission.py verdict-record --red-names 照计划里的键记红、撤掉同一把键原有的绿（拒算的没有键，verdict-record 不记），下一趟这几条现判、原因以「上次判红」起头。
#   给了 <清单文件>（这一趟 account_test_results 认成了）：同一趟判绿的经 green_names_of_pass 取名单、交同一次 verdict-record 照键记绿（文件头「复用」：
#   只认有明确结局行的；编不过、一条结局行都没有的，名单是空的，一条都不记）；没给的（认不成，或弄坏开关 *-red-withholds-green 下调用处不给）判绿的一条都不记。
#   打「· 判红记库：…」与「· 判绿记库（<格名> 这一趟判红）：…」各一行；记不下只打一行「!」（这一格已经判红），复用哨兵报的失灵照 plan_failure_verdict 打出路。
#   弄坏开关 red-not-recorded 下不交红的（改前的样子：门禁判红的不撤同一把键原有的绿），判绿的照记
record_red_tests() {
  local kind="$1" plan_file="$2" plan_output="$3" red_rows="$4" cell="$5" how="$6" accounting="${7:-}" red_names="$crash_point_replay_scratch_directory/red-names.$5"
  local green_names="$crash_point_replay_scratch_directory/red-names.$5.green" red_count=0 green_count=0 record_exit
  local -a red_option=()
  : >"$red_names"; : >"$green_names"
  if [[ "$crash_point_replay_break" != red-not-recorded && -s "$red_rows" ]]; then
    awk -F'\t' 'FNR == NR { if (NF >= 7 && ($2 == "judge" || $2 == "reuse" || $2 == "refused")) name[$5 SUBSEP $6] = $1; next }
      (($1 SUBSEP $2) in name) && !seen[name[$1 SUBSEP $2]]++ { print name[$1 SUBSEP $2] }' "$plan_output" "$red_rows" >"$red_names"
    red_count="$(grep -c . "$red_names")"
    if (( red_count == 0 )); then
      echo "  · 判红记库：红的 $(grep -c . "$red_rows") 条测试函数在这一趟的计划里一条都对不上，不记"
    else
      red_option=(--red-names "$red_names")
    fi
  fi
  if [[ -n "$accounting" ]] && green_names_of_pass "$plan_output" "$accounting" "$green_names" "判绿记库（$cell 这一趟判红）"; then
    green_count="$(grep -c . "$green_names")"
    (( green_count > 0 )) || { echo "  · 判绿记库（$cell 这一趟判红）：没有可记的（判绿的都不认、一条都没判绿，或编不过、一条结局行都没有）"; }
  fi
  (( red_count + green_count > 0 )) || return 0
  if python3 "$crash_point_replay_admission_module" verdict-record "$ROOT" "$kind" "$plan_file" "$green_names" ${red_option[@]+"${red_option[@]}"} --how "$how" \
       >"$red_names.record" 2>&1; then record_exit=0; else record_exit=$?; fi
  if ((record_exit == 0)); then
    (( red_count > 0 )) && echo "  · 判红记库：${red_count} 条记红、撤掉同一把键原有的绿，下一趟照现判（$(tail -n 1 "$red_names.record")）"
    (( green_count > 0 )) && echo "  · 判绿记库（$cell 这一趟判红）：判绿的 ${green_count} 条照键记库（$(tail -n 1 "$red_names.record")）"
  else
    echo "  ! 判红记库：admission.py verdict-record 退 $record_exit（$(tail -n 2 "$red_names.record" | tr '\n' ' ' | cut -c1-300)），红的没记下、同一把键原有的绿还在；判绿的 ${green_count} 条也没记下，下一趟照跑"
    # 这一格已经判红：记库那一步 prune-server 不可用也照红算（crash_point_replay_red_regardless_cells），不当只是不可用
    plan_failure_verdict "$cell" verdict-record "$record_exit" "$red_names.record" || crash_point_replay_red_regardless_cells+=("$cell")
  fi
  return 0
}

# record_cpu_judging_pass <种类> <计划文件> <计划输出> <清单文件> <格名> <跑法>：多机配置开了 GPU、那一趟 cargo test 退 0 而日志里有判段判在 CPU 的行时
#   （文件头「多机配置开了 GPU」：这一格照旧判红），同一趟的判定照记，记法同判红那一趟（record_red_tests：红的与绿的交同一次 verdict-record）。
#   <清单文件> 是那一趟 account_test_results / account_feature_pass 认出的：开了 GPU 不带 --cpu-judging-recordable，判在 CPU 的记号认作谁打的，那几条已经不在判绿里。
#   判在 CPU 的记号（清单里 marker … cpu-judging 那几行）分两层认：complete 是 1、认作谁打的恰好一条的（带 --test-threads 1 的一段是打它时正在跑的那一条，
#   几条同时跑的一段是打它时只剩那一条还没出结果），归到那一条测试函数：照计划里的键记红、撤掉同一把键原有的绿，下一趟现判、原因以「上次判红」起头；
#   别的（几条同时在跑、这一段认不全）归不到测试函数、只归到它所在的测试二进制：那个二进制里判绿的这一趟一条都不记（清单里改成 withheld、写明为什么），
#   别的二进制判绿的照键记绿。打一行「· <格名>：判段判在 CPU 的记号 N 处：…」报两层各几处、归到二进制的是哪几个。
#   一处记号都没认出（判在 CPU 的行不在哪个测试二进制那一段里）的，归不到谁，这一趟一条都不记、打一行「!」。
#   调用处的弄坏开关 plain-pass-cpu-judging-records-nothing、feature-pass-cpu-judging-records-nothing 下不调它（改前的样子：这一趟一条都不记）
record_cpu_judging_pass() {
  local kind="$1" plan_file="$2" plan_output="$3" accounting="$4" cell="$5" how="$6"
  local red_rows="$crash_point_replay_scratch_directory/cpu-judging-red-rows.$5" kept="$crash_point_replay_scratch_directory/cpu-judging-accounting.$5"
  local function_count=0 binary_count=0 binaries=""
  : >"$red_rows"
  if ! awk -F'\t' -v red_rows="$red_rows" -v counts="$kept.counts" 'BEGIN { OFS = "\t" }
      FNR == NR {
        if ($1 == "marker" && $3 == "cpu-judging") {
          if ($4 == 1 && NF == 6) { functions++; if (!seen[$2 SUBSEP $6]++) print $2, $6 >> red_rows }
          else { located++; if (!($2 in binary)) { binary[$2] = 1; order[++binaries] = $2 } }
        }
        next
      }
      $1 == "green" && ($2 in binary) { print "withheld", $2, $3, "这个测试二进制打了判段判在 CPU 的一行、归不到哪一条测试函数（几条同时在跑，或这一段认不全），整个二进制判绿的这一趟都不记"; next }
      { print }
      END { listed = ""; for (index_in_order = 1; index_in_order <= binaries; index_in_order++) listed = listed (index_in_order > 1 ? "、" : "") order[index_in_order]
            printf "%d\t%d\t%s\n", functions, located, listed > counts }' "$accounting" "$accounting" >"$kept"; then
    echo "  ! $cell：判段判在 CPU 的记号分不了层（awk 没成），这一趟一条都不记，下一趟照跑"
    return 0
  fi
  IFS=$'\t' read -r function_count binary_count binaries <"$kept.counts" || :
  if (( function_count + binary_count == 0 )); then
    echo "  ! $cell：日志里有判段判在 CPU 的行，逐条认结局却一处记号都没认出（那几行不在哪个测试二进制那一段里），归不到谁，这一趟一条都不记，下一趟照跑"
    return 0
  fi
  echo "  · $cell：判段判在 CPU 的记号 $((function_count + binary_count)) 处：归到测试函数的 ${function_count} 处（那几条照键记红、撤绿，下一趟现判），归不到测试函数、只归到测试二进制的 ${binary_count} 处（${binaries:-无}：这几个二进制里判绿的这一趟都不记）；别的判绿的照键记绿"
  record_red_tests "$kind" "$plan_file" "$plan_output" "$red_rows" "$cell" "$how" "$kept"
}

# not_run_markers_of <清单文件> <落进的文件>：account_test_results 认出的 NOT_RUN_ 记号，一处一行「<源文件><TAB><complete：1|0><TAB><那一截>[<TAB><认作谁打的>…]」
not_run_markers_of() {
  awk -F'\t' 'BEGIN { OFS = "\t" } $1 == "marker" && $3 == "not-run" { line = $2 OFS $4 OFS $5; for (i = 6; i <= NF; i++) line = line OFS $i; print line }' "$1" >"$2"
}

# supplement_plain_not_run <带特性那一趟的计划输出> <它的清单文件> <计划算没算成：1|0>：不带特性那一趟打了 NOT_RUN_ 的测试函数（crash_point_replay_plain_not_run_file）
# 带特性那一趟补判到没有（文件头「多机配置开了 GPU」）：认作它打的每一条在这一趟判绿（清单文件里的 green）、或这一组特性下有键相同的判定复用，才算判到。
# 一处一行「<1|0><TAB><哪几条怎么判到的，或为什么没判到>」落进 crash_point_replay_supplement_file，与 crash_point_replay_plain_not_run_file 逐行对应
supplement_plain_not_run() {
  python3 - "$crash_point_replay_plain_not_run_file" "$1" "$2" "$3" >"$crash_point_replay_supplement_file" <<'PY' || : >"$crash_point_replay_supplement_file"
import sys
markers_file, plan_output, accounting, plan_ok = sys.argv[1:5]
decisions = {}
if plan_ok == "1":
    for line in open(plan_output, encoding="utf-8", errors="replace"):
        columns = line.rstrip("\n").split("\t")
        if len(columns) >= 7:
            decisions[(columns[4], columns[5])] = columns[1]
green = set()
for line in open(accounting, encoding="utf-8", errors="replace"):
    columns = line.rstrip("\n").split("\t")
    if len(columns) >= 3 and columns[0] == "green":
        green.add((columns[1], columns[2]))
for line in open(markers_file, encoding="utf-8", errors="replace"):
    columns = line.rstrip("\n").split("\t")
    source, complete, candidates = columns[0], (columns[1] if len(columns) > 1 else "0"), [name for name in columns[3:] if name]
    if complete != "1" or not candidates:
        print("0\t认不出是这一段哪一条测试函数打的，补判不了")
        continue
    judged, missing = [], []
    for name in candidates:
        decision = decisions.get((source, name))
        if decision == "reuse":
            judged.append(f"{source}::{name} 键相同、复用")
        elif (source, name) in green and decision in (None, "judge", "refused"):
            judged.append(f"{source}::{name} 判绿")
        elif decision == "not-compiled":
            missing.append(f"{source}::{name} 这一组特性下不编")
        else:
            missing.append(f"{source}::{name} 这一趟没跑到或没判绿")
    print(("0\t" + "、".join(missing)) if missing else ("1\t" + "、".join(judged)))
PY
}

# report_plain_pass_not_run <带特性那一格：ran|red|空（没点它）>：不带特性那一趟打了 NOT_RUN_ 的，补判到的打一行「·」，没补判到的逐条报本次未跑
# （文件头「多机配置开了 GPU」），条数放进 crash_point_replay_plain_not_run_after
report_plain_pass_not_run() {
  local where complete text _candidates verdict why reason_without="" index=0
  local -a verdicts=()
  crash_point_replay_plain_not_run_after=0
  [[ -s "$crash_point_replay_plain_not_run_file" ]] || return 0
  case "$1" in
    "") reason_without="这一趟没跑带特性那一格（--check 没点 $FEATURE_GATED_TESTS_CELL_NAME）" ;;
    red) reason_without="带特性那一格判红" ;;
    *)
      if [[ -f "$crash_point_replay_supplement_file" ]]; then mapfile -t verdicts <"$crash_point_replay_supplement_file"
      elif [[ "$feature_cell_state" == 无对象可判 ]]; then reason_without="验证包 tests/ 下没有挂特性的测试文件，带特性那一趟没起"
      else reason_without="带特性那一趟没交出补判的结论"; fi ;;
  esac
  while IFS=$'\t' read -r where complete text _candidates; do
    verdict="${verdicts[index]:-}"
    index=$((index + 1))
    if [[ -z "$reason_without" && "${verdict%%$'\t'*}" == 1 ]]; then
      echo "  · 不带特性那一趟（${where}）报的本次未跑由带特性那一趟补判到了：${text}（${verdict#*$'\t'}）"
      continue
    fi
    why="$reason_without"
    if [[ -z "$why" && "$verdict" == *$'\t'* ]]; then why="${verdict#*$'\t'}"; fi
    [[ -n "$why" ]] || why="补判的结论没取出来"
    crash_point_replay_plain_not_run_after=$((crash_point_replay_plain_not_run_after + 1))
    crash_point_replay_report_not_run "验证包不带特性那一趟（${where}）报了本次未跑：${text}；带特性那一趟没补判到（${why}），这一趟不算跑过"
  done <"$crash_point_replay_plain_not_run_file"
  return 0
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

# report_red_test_targets <cargo 的整份输出> <格名> [<红的测试函数落进的文件>]：按条复用两趟判红时逐个红的测试目标抄出 FAILED 的测试函数名与 panic 段，整份输出拷到 ${TMPDIR:-/tmp} 下留着
# （文件头「复用」之后「按条复用两趟判红时抄出红的测试」那一段）。弄坏开关 red-tail-only 下照改前什么都不抄、不留，只剩调用处的尾巴与 FAILED 那一条 grep。
# 给了第三个参数：逐个红的测试目标里 FAILED 的测试函数一条一行「<源文件><TAB><libtest 全名>」落进那份文件（record_red_tests 照它记红；red-tail-only 下照落）。
report_red_test_targets() {
  local log="$1" cell="$2" red_rows="${3:-}" kept_log copied=1
  [[ "$crash_point_replay_break" == red-tail-only ]] && copied=0
  if ! python3 - "$crash_point_replay_stage_repository/scripts/gate/lib_libtest_result_lines.py" "$log" "$cell" "$red_rows" "$copied" <<'PY_RED_TEST_TARGETS'
import importlib.util
import re
import sys

sys.dont_write_bytecode = True
library_path, log_path, cell, red_rows_path, copied = sys.argv[1:6]
specification = importlib.util.spec_from_file_location("lib_libtest_result_lines_for_red_test_targets", library_path)
libtest = importlib.util.module_from_spec(specification)
specification.loader.exec_module(libtest)
PANIC_START = re.compile(r"^thread '(?P<thread>[^']*)' panicked at ")
FAILURE_LIST_ENTRY = re.compile(r"^    (?P<name>\S+)\s*$")
SEGMENT_LINES = 40          # 一段 panic 至多抄几行
SEGMENTS_PER_TARGET = 8     # 一个测试目标至多抄几段 panic
TAIL_WITHOUT_SUMMARY = 15   # 没有汇总行的那一段抄末几行
LINE_CHARACTERS = 400       # 一行至多抄几个字


def shown(line):
    return line if len(line) <= LINE_CHARACTERS else line[:LINE_CHARACTERS] + "…（这一行截了）"


with open(log_path, encoding="utf-8", errors="replace") as handle:
    text = handle.read()
sections = []
for number, line in enumerate(text.split("\n"), 1):
    running = libtest.RUNNING_LINE_FORM.match(line)
    doctests = libtest.DOCTESTS_LINE_FORM.match(line)
    if running or doctests:
        sections.append([running.group("source") if running else f"doctest:{doctests.group('package')}", number, []])
    elif sections:
        sections[-1][2].append((number, line))
red = []
for source, start, numbered in sections:
    summary, failed, pending, in_failure_list = None, [], None, False
    for _number, line in numbered:
        parsed = libtest.result_line(line)
        if parsed is not None:
            pending = None
            if parsed.outcome == "FAILED":
                failed.append(parsed.full_name)
            continue
        if summary is None and libtest.summary_line(line) is not None:
            summary = libtest.summary_line(line)
            in_failure_list = False
            continue
        started = libtest.SCATTERED_START_FORM.match(line)
        if started is not None and not line.startswith("test result: "):
            pending = started.group("name")
            continue
        alone = libtest.STANDALONE_OUTCOME_FORM.match(line)
        if alone is not None and pending is not None:
            if alone.group("outcome") == "FAILED":
                failed.append(pending)
            pending = None
            continue
        if line.rstrip() == "failures:":
            in_failure_list = True
            continue
        if in_failure_list:
            entry = FAILURE_LIST_ENTRY.match(line)
            if entry is not None:
                failed.append(entry.group("name"))
            elif line.strip():
                in_failure_list = False
    failed = list(dict.fromkeys(failed))
    if summary is not None and summary.outcome != "FAILED" and summary.failed == 0 and not failed:
        continue
    segments, segment = [], None
    for number, line in numbered:
        if PANIC_START.match(line):
            segment = [number, [line]]
            segments.append(segment)
        elif segment is not None and (line.startswith("test ") or line.startswith("---- ") or line.rstrip() == "failures:"
                                      or libtest.STANDALONE_OUTCOME_FORM.match(line)):
            segment = None
        elif segment is not None:
            segment[1].append(line)
    red.append((source, start, numbered[-1][0] if numbered else start, summary, failed, segments, numbered, pending))
if red_rows_path:
    with open(red_rows_path, "w", encoding="utf-8", errors="surrogateescape") as handle:
        handle.write("".join(f"{target[0]}\t{name}\n" for target in red for name in target[4]))
if copied != "1":
    sys.exit(0)
failed_total = sum(len(target[4]) for target in red)
print(f"  · {cell}：红的测试目标 {len(red)} 个、FAILED 的测试函数 {failed_total} 条（逐个目标抄在下面：FAILED 的名字与 panic 段，"
      "取自这一趟 cargo 的整份输出，不只是上面的尾巴）")
for source, start, end, summary, failed, segments, numbered, pending in red:
    summary_text = (f"汇总行 {summary.outcome}. {summary.passed} passed; {summary.failed} failed" if summary is not None
                    else "没有汇总行：测试二进制没跑完（被信号杀、撞了上限，或中途退了）")
    print(f"    红的测试目标 {source}（日志第 {start}–{end} 行；{summary_text}）：")
    for name in failed:
        print(f"      FAILED {name}")
    if summary is not None and summary.failed and not failed:
        print(f"      （汇总行报 {summary.failed} 条失败，名字一条都没认出：结果行被测试自己的输出打散了，看整份输出）")
    for number, lines in segments[:SEGMENTS_PER_TARGET]:
        thread = PANIC_START.match(lines[0]).group("thread")
        while lines and not lines[-1].strip():
            lines.pop()
        print(f"      panic 段（thread '{thread}'，日志第 {number} 行起 {len(lines)} 行{'，抄前 ' + str(SEGMENT_LINES) + ' 行' if len(lines) > SEGMENT_LINES else ''}）：")
        for line in lines[:SEGMENT_LINES]:
            print(f"        {shown(line)}")
    if len(segments) > SEGMENTS_PER_TARGET:
        print(f"      …另有 {len(segments) - SEGMENTS_PER_TARGET} 段 panic 没抄，看整份输出")
    if summary is None and pending is not None:
        print(f"      跑到一半的测试函数 {pending}（只打了「test {pending} ... 」起头，没有结局）")
    if summary is None:
        print(f"      这一段末 {min(TAIL_WITHOUT_SUMMARY, len(numbered))} 行：")
        for _number, line in numbered[-TAIL_WITHOUT_SUMMARY:]:
            print(f"        {shown(line)}")
PY_RED_TEST_TARGETS
  then
    echo "    ! $cell：抄红的测试函数名与 panic 段那一步没成（上面是 python 的报错）；整份输出看下一行的路径"
  fi
  (( copied )) || return 0
  kept_log="$(mktemp "${TMPDIR:-/tmp}/crash-point-replay-$cell.XXXXXX.log")" || kept_log=""
  if [[ -n "$kept_log" ]] && cp -- "$log" "$kept_log"; then
    crash_point_replay_kept_logs+=("$kept_log"$'\t'"$(sha256sum < "$log" | cut -d' ' -f1)")
    echo "    · $cell：这一趟 cargo 的整份输出留在 $kept_log（这一道的草稿目录跑完就删，这一份不删；看完自己删）"
  else
    [[ -n "$kept_log" ]] && rm -f -- "$kept_log"
    echo "    ! $cell：这一趟 cargo 的整份输出没留下（拷不到 ${TMPDIR:-/tmp} 下）；上面抄出的名字与 panic 段是这一趟仅有的记录"
  fi
}

# report_pass_outcome <格名> <哪一趟> <退出码> <整份输出>：一趟 cargo 一收尾就往标准输出打这一趟的结局（文件头「每一趟当场写结局」）：退出码、test result 汇总行
# （几行、合计通过 / 失败 / ignored、FAILED 的几行）、FAILED 的测试函数（结果行「test <名> ... FAILED」与 libtest 末尾 failures: 名单取并集，至多列 20 条）、整份输出的路径。
# 不判、不改这一格的结论。弄坏开关 pass-outcome-at-cell-end 下照改前不当场打：先记进 crash_point_replay_deferred_pass_outcomes，整格收尾才由 flush_deferred_pass_outcomes 打
report_pass_outcome() {
  local cell="$1" pass="$2" pass_exit="$3" log="$4" line summary_count=0 passed_total=0 failed_total=0 ignored_total=0 failed_summary_count=0 names_text outcome
  local -a failed_names=()
  if [[ -f "$log" ]]; then
    while IFS= read -r line; do
      summary_count=$((summary_count + 1))
      if [[ "$line" =~ ([0-9]+)\ passed\;\ ([0-9]+)\ failed\;\ ([0-9]+)\ ignored ]]; then
        passed_total=$((passed_total + BASH_REMATCH[1])); failed_total=$((failed_total + BASH_REMATCH[2])); ignored_total=$((ignored_total + BASH_REMATCH[3]))
      fi
      [[ "$line" == "test result: FAILED"* ]] && failed_summary_count=$((failed_summary_count + 1))
    done < <(grep '^test result: ' "$log")
    mapfile -t failed_names < <(awk '
      /^test .* \.\.\. FAILED$/ { name = $0; sub(/^test /, "", name); sub(/ \.\.\. FAILED$/, "", name); print name; next }
      /^failures:$/ { in_list = 1; next }
      in_list && /^    [^ ]+$/ { name = $0; sub(/^    /, "", name); print name; next }
      in_list && /^[^ ]/ { in_list = 0 }' "$log" | awk '!seen[$0]++')
  fi
  names_text=""
  if (( ${#failed_names[@]} > 0 )); then
    names_text="：$(printf '%s、' "${failed_names[@]:0:20}")"
    names_text="${names_text%、}"
    (( ${#failed_names[@]} > 20 )) && names_text+="（另有 $(( ${#failed_names[@]} - 20 )) 条，看整份输出）"
  fi
  if (( summary_count == 0 )); then
    outcome="test result 汇总行 0 行（没跑完：被信号杀、撞了上限，或中途退了；一条都没起的也是 0 行）"
  else
    outcome="test result 汇总行 ${summary_count} 行（合计通过 ${passed_total}、失败 ${failed_total}、ignored ${ignored_total}；FAILED 的汇总行 ${failed_summary_count} 行）"
  fi
  line="  · ${cell}：${pass}收尾，当场写结局：cargo 退 ${pass_exit}；${outcome}；FAILED 的测试函数 ${#failed_names[@]} 条${names_text}；整份输出：${log}"
  if [[ "$crash_point_replay_break" == pass-outcome-at-cell-end ]]; then
    printf '%s\n' "$line" >> "$crash_point_replay_deferred_pass_outcomes"
    return 0
  fi
  printf '%s\n' "$line"
}

# flush_deferred_pass_outcomes：整格收尾时打弄坏开关 pass-outcome-at-cell-end 下记下的那几趟结局（改前的样子：等整格收尾才写），打完清空；没开开关时这份是空的、什么都不打
flush_deferred_pass_outcomes() {
  [[ -s "$crash_point_replay_deferred_pass_outcomes" ]] || return 0
  cat -- "$crash_point_replay_deferred_pass_outcomes"
  : > "$crash_point_replay_deferred_pass_outcomes"
}

# crash_point_replay_test_runner <数组名> [gpu]：按条复用那一趟起测试用的命令词放进数组（文件头「按条复用两趟按测试目标并行起」）：
# scripts/gate/test-targets-parallel.py --jobs <每台默认重活线程数> --，或照旧 cargo test（样本档、取不到线程数或那份脚本、第二个参数是 gpu）
crash_point_replay_test_runner() {
  local -n runner_words="$1"
  local jobs="" library="$crash_point_replay_stage_repository/scripts/hosts/heavy-threads.sh" parallel="$crash_point_replay_stage_repository/scripts/gate/test-targets-parallel.py"
  runner_words=(cargo test)
  [[ "${2:-}" != gpu && "$crash_point_replay_sample_mode" != 1 && -f "$parallel" && -f "$library" ]] || return 0
  jobs="$( (source "$library" && heavy_thread_default) 2>/dev/null)"
  [[ "$jobs" =~ ^[1-9][0-9]*$ ]] || return 0
  runner_words=(python3 "$parallel" --jobs "$jobs" --)
}

# run_checker_package_tests <日志> [<libtest 参数…>]：只跑不标 ignored 的用例（--tests 连 src/bin/ 的内联单测一起带上），键相同的那几条经 -- --exact --skip 跳过
# （参数由调用方从计划里取，skip_arguments_of_plan）；cargo 的整段输出进日志；这一趟不写发现日志，清掉调用方的 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE。
run_checker_package_tests() {
  local log="$1"
  shift
  local -a runner=()
  crash_point_replay_test_runner runner
  env -u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
    "${runner[@]}" --release --no-fail-fast -p singlefs-verification --lib --tests -- --nocapture "$@" 2>&1 \
    | tee "$log" \
    | { grep --line-buffered '^CRASH_POINT_REPLAY_PROGRESS ' || true; } \
    | sed -u 's/^/    /'
  return "${PIPESTATUS[0]}"
}

# read_crash_case_command <键>：准入模块 crash-case-command 交出的这一条的命令与环境（以 NUL 分隔）读进 case_machine_cores、
# case_threads、case_threads_origin（这三个只打进开跑那一行）与数组 case_command。
# 交不出、写法不对返回 1，原因放进 case_command_problem。
read_crash_case_command() {
  local command_file="$crash_point_replay_scratch_directory/command.${1#crash-case:}" command_output
  # 样本档里准入模块照样本自己的找法找多机配置（样本根的 multi-host.env），不读调用方环境里 SINGLEFS_MULTI_HOST_CONFIG 指的那一份（文件头「多机配置开了 GPU」）
  local -a configuration_unset=()
  if [[ "$crash_point_replay_sample_mode" == 1 && "$crash_point_replay_break" != sample-caller-configuration-kept ]]; then
    configuration_unset=(-u SINGLEFS_MULTI_HOST_CONFIG)
  fi
  if ! env ${configuration_unset[@]+"${configuration_unset[@]}"} python3 "$crash_point_replay_admission_module" crash-case-command "$ROOT" "$1" > "$command_file"; then
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
    | { grep --line-buffered -E '^CRASH_POINT_REPLAY_PROGRESS |^CRASH_AMPLIFICATION_BLOCK ' || true; } \
    | sed -u 's/^/    /'
  return "${PIPESTATUS[0]}"
}

# report_crash_case_findings <发现日志> <用例日志>：逐节打发现表（行格式与节的写法以 crates/singlefs-verification/src/crash.rs 的发现日志为准：
# 字段制表符分，第一个字段是行的种类，其余 key=value）。每一处不对各打一行「问题：…」，有问题返回 1；发现日志不在又不该有时返回 0。
report_crash_case_findings() {
  local findings_file="$1" log_file="$2" stdout_findings_lines
  stdout_findings_lines="$(grep -c '^CRASH_POINT_REPLAY_FINDINGS ' "$log_file" 2>/dev/null)"
  stdout_findings_lines="${stdout_findings_lines:-0}"
  if [[ ! -s "$findings_file" ]]; then
    if [[ "$stdout_findings_lines" == 0 ]]; then
      echo "    发现日志：没有（$findings_file 不在或是空的，用例日志里也没有 CRASH_POINT_REPLAY_FINDINGS 行：这条用例没走读 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 的枚举入口）"
      return 0
    fi
    echo "    发现日志：$findings_file 不在或是空的"
    echo "      问题：用例日志里有 ${stdout_findings_lines} 行 CRASH_POINT_REPLAY_FINDINGS（枚举跑完了 ${stdout_findings_lines} 趟），发现日志却一节都没写：SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 没传进用例"
    return 1
  fi
  echo "    发现日志：$findings_file（用例日志里 CRASH_POINT_REPLAY_FINDINGS ${stdout_findings_lines} 行）"
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

# registered_test_functions：登记的崩溃枚举用例与放量分项各落到哪个「测试目标<TAB>用例函数」，一行一条（文件头「复用」）：
# 登记的崩溃枚举用例取 admission.py crash-cases 交的那几行（第三、四列），放量分项取 scripts/crash/lib_crash_point_replay_items.py 的 pipeline_items 读 crash-amplification.sh 的 ITEMS
# （放量入口不在被判的根里，就是空）。按条复用那一趟用 --skip 把它们排掉，交按块那一路（run_registered_crash_cases）。弄坏开关 amplification-items-not-skipped 下不取放量分项那一半
registered_test_functions() {
  local library="$crash_point_replay_stage_repository/scripts/crash/lib_crash_point_replay_items.py"
  printf '%s\n' "$crash_case_listing" | awk -F'\t' 'NF >= 4 { printf "%s\t%s\n", $3, $4 }'
  [[ "$crash_point_replay_break" == amplification-items-not-skipped || ! -f "$library" ]] && return 0
  python3 - "$library" "$ROOT" <<'PY_AMPLIFICATION_ITEMS'
import importlib.util
import sys

sys.dont_write_bytecode = True
library_path, root = sys.argv[1:3]
specification = importlib.util.spec_from_file_location("lib_crash_point_replay_items_for_the_skip_list", library_path)
library = importlib.util.module_from_spec(specification)
specification.loader.exec_module(library)
for target, function in sorted(library.pipeline_items(root).values()):
    print(f"{target}\t{function}")
PY_AMPLIFICATION_ITEMS
}

# drop_registered_rows_from_plan <计划输出>：登记的崩溃枚举用例与放量分项不在按条复用这一趟里（它们走按块那一路），从 test-plan 的逐条行里去掉：
# 目标根文件是 tests/<测试目标>.rs、libtest 全名（去掉模块路径）是登记的用例函数的行；末行 PLAN 照剩下的行重数
drop_registered_rows_from_plan() {
  local plan_output="$1" filtered="$1.registered-dropped"
  [[ -s "$crash_point_replay_registered_pairs_file" ]] || return 0
  awk -F'\t' '
    FNR == NR { registered["tests/" $1 ".rs" SUBSEP $2] = 1; next }
    /^PLAN / { plan_line = $0; next }
    NF >= 7 {
      name = $6; sub(/^.*::/, "", name)
      if (($5 SUBSEP name) in registered) next
      count[$2]++
    }
    { print }
    END {
      ignored = ""
      if (match(plan_line, / ignored=[0-9]+/)) ignored = substr(plan_line, RSTART, RLENGTH)
      printf "PLAN judge=%d reuse=%d refused=%d not-compiled=%d%s\n", count["judge"], count["reuse"], count["refused"], count["not-compiled"], ignored
    }' "$crash_point_replay_registered_pairs_file" "$plan_output" >"$filtered" && mv -- "$filtered" "$plan_output"
}

# run_package_test_functions：验证包里全部测试函数按单元键复用（文件头「复用」），登记的崩溃枚举用例与放量分项经 --skip 排掉。判红打出路、返回 1；判绿把现判、复用、拒算与每个测试二进制
# 那一行 test result 的 passed / ignored 加总进 package_pass_report。一条都不用现判就不起 cargo（package_pass_ran=0）。
run_package_test_functions() {
  local pass_log result_lines passed_count ignored_count passed_total=0 ignored_total=0 binary_count=0 plan_ok=1 accounting_ok=0
  local kind package_pass_exit=0
  local -a red_pass_accounting=()
  kind="$(crash_point_replay_verdict_kind "$crash_point_replay_stage_file_name" "$CRASH_CASE_REPLAY_CELL_NAME")"
  local plan_file="$crash_point_replay_scratch_directory/plain.plan" plan_output="$crash_point_replay_scratch_directory/plain.plan.out"
  local accounting="$crash_point_replay_scratch_directory/plain.accounting"
  pass_log="$crash_point_replay_plain_pass_log"
  package_pass_ran=1
  : >"$pass_log"
  : >"$crash_point_replay_plain_not_run_file"
  plan_skip_arguments=()
  if plan_tests "$kind" "$plan_file" "$plan_output"; then
    skip_arguments_of_plan "$plan_output"
    echo "  · 按条复用：验证包 $((plan_judge + plan_reuse + plan_refused)) 条编得进的测试函数，$(plan_report "$plan_output")；这一组特性下不编 $plan_not_compiled 条；登记的崩溃枚举用例与放量分项 $(grep -c . "$crash_point_replay_registered_pairs_file") 条经 --skip 排掉、走按块那一路"
    plan_listing "$plan_output"
  else
    plan_ok=0
    echo "  ! 按条复用算不出（$plan_problem）：这一趟全部现判、一条都不记库"
    ((${#plan_failure[@]})) && { plan_failure_verdict "$CRASH_CASE_REPLAY_CELL_NAME" "${plan_failure[@]}" || package_pass_plan_red=1; }
    skip_arguments_of_plan -
  fi
  crash_point_replay_plain_plan_ok="$plan_ok"
  # 计划那一步 prune-server 不可用（plan_failure_verdict 记了这一格）：不起 cargo，这一格就此收尾
  if crash_point_replay_unavailable_only "$CRASH_CASE_REPLAY_CELL_NAME"; then package_pass_ran=0; return 1; fi   # 出口 4 那几格读这一趟的行时要（文件头「出口 4 那五格」）
  if (( plan_ok && plan_judge + plan_refused == 0 )); then
    package_pass_ran=0
    package_pass_report="验证包 $plan_reuse 条测试函数都有键相同的判定，全部复用，这一趟没起 cargo（现判 0、复用 $plan_reuse、拒算 0、本次未跑 0）"
  elif run_checker_package_tests "$pass_log" ${plan_skip_arguments[@]+"${plan_skip_arguments[@]}"}; then
    report_pass_outcome "$CRASH_CASE_REPLAY_CELL_NAME" "不带特性那一趟" 0 "$pass_log"
  else
    package_pass_exit=$?
    report_pass_outcome "$CRASH_CASE_REPLAY_CELL_NAME" "不带特性那一趟" "$package_pass_exit" "$pass_log"
    tail -40 "$pass_log"
    # --no-fail-fast：一个测试二进制红了其余照跑，红的用例与测试目标在这里一次列全
    grep -E '^test .* FAILED$|^error: test failed, to rerun pass' "$pass_log" | sed 's/^/    /' || true
    (( plan_ok )) && failed_tests_changed_units "$plan_output" "$pass_log"
    report_red_test_targets "$pass_log" "$CRASH_CASE_REPLAY_CELL_NAME" "$crash_point_replay_scratch_directory/red-rows.$CRASH_CASE_REPLAY_CELL_NAME"
    # 同一趟判绿的照键记绿（文件头「复用」）：弄坏开关 plain-pass-red-withholds-green 下照改前不认、不记
    if (( plan_ok )) && [[ "$crash_point_replay_break" != plain-pass-red-withholds-green ]] && account_test_results "$pass_log" 0 "$accounting"; then
      red_pass_accounting=("$accounting")
    fi
    (( plan_ok )) && record_red_tests "$kind" "$plan_file" "$plan_output" "$crash_point_replay_scratch_directory/red-rows.$CRASH_CASE_REPLAY_CELL_NAME" \
      "$CRASH_CASE_REPLAY_CELL_NAME" "cargo test --release -p singlefs-verification --lib --tests（不带特性那一趟）" ${red_pass_accounting[@]+"${red_pass_accounting[@]}"}
    echo "  ✗ 验证包判红（上面是 cargo test 的尾部、全部红的用例与测试目标，「红的」那几行是各自变了的单元；键相同而复用的那几条这一趟没跑）"
    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-verification -- --nocapture（只看一个测试二进制加 --test <名>）；"
    echo "                断言消息里是第一处对不上的计数或违例；改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
    return 1
  fi
  if (( package_pass_ran )); then
    result_lines="$(sed -n 's/^test result: ok\. \([0-9]*\) passed; 0 failed; \([0-9]*\) ignored;.*/\1 \2/p' "$pass_log")"
    while read -r passed_count ignored_count; do
      [[ -n "${passed_count:-}" ]] || continue
      passed_total=$((passed_total + passed_count)); ignored_total=$((ignored_total + ignored_count)); binary_count=$((binary_count + 1))
    done <<< "$result_lines"
    if (( passed_total == 0 )); then
      echo "  ✗ 验证包里的测试函数跑过了，却读不到 cargo 的 test result 行，或要现判的用例一条都没通过：扫到 0 条不是通过"
      echo "     → 怎么办：cargo test --release -p singlefs-verification -- --list 看这个包里还剩几条测试函数；"
      echo "                一条都没有，就是被整批标了 ignored 或删掉了，补回来。"
      return 1
    fi
  fi
  # 配置开了 GPU（文件头「多机配置开了 GPU」）：判段判在 CPU 的判红，同一趟的判定照记；用例报的本次未跑等带特性那一趟补判之后再逐条报，不判红、不算通过
  local cpu_judging=() plain_where plain_line
  if (( crash_point_replay_gpu_configured )) && [[ "$crash_point_replay_break" != cpu-judging-accepted ]]; then
    mapfile -t cpu_judging < <(cpu_judging_lines "$pass_log")
    if (( ${#cpu_judging[@]} > 0 )); then
      printf '      %s\n' "${cpu_judging[@]:0:20}"
      # 判在 CPU 归得到的那几条记红、归不到的整个二进制不记、别的判绿的照键记绿（record_cpu_judging_pass）：弄坏开关 plain-pass-cpu-judging-records-nothing 下照改前一条都不记
      if (( plan_ok && package_pass_ran )) && [[ "$crash_point_replay_break" != plain-pass-cpu-judging-records-nothing ]] && account_test_results "$pass_log" 0 "$accounting"; then
        record_cpu_judging_pass "$kind" "$plan_file" "$plan_output" "$accounting" "$CRASH_CASE_REPLAY_CELL_NAME" \
          "cargo test --release -p singlefs-verification --lib --tests（不带特性那一趟）"
      fi
      echo "  ✗ 验证包不带 gpu 特性那一趟：多机配置 $crash_point_replay_configuration_file 写了 ENABLE_GPU=1，日志里有 ${#cpu_judging[@]} 处判段判在 CPU（上面列了前 20 处），配了 GPU 不退回 CPU"
      echo "     → 怎么办：不带 gpu 特性那一趟遇到 ENABLE_GPU=1 打一行 NOT_RUN_<原因> test=<用例> reason=<一句> 报本次未跑、不判（crates 那一侧改），判归带特性那一趟（feature-gated-tests 格）；改完只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME"
      return 1
    fi
  fi
  # 逐条认结局与记号（文件头「复用」）；这一趟打了 NOT_RUN_ 的测试函数落进 crash_point_replay_plain_not_run_file，等带特性那一趟补判完再报（文件头「多机配置开了 GPU」）。
  # 认不成时退回按行收 NOT_RUN_（不知道是哪一条打的，补判不了）
  if (( package_pass_ran )) && account_test_results "$pass_log" 0 "$accounting"; then accounting_ok=1; fi
  if [[ "$crash_point_replay_break" == plain-pass-not-run-unreported ]]; then
    :
  elif (( accounting_ok )); then
    not_run_markers_of "$accounting" "$crash_point_replay_plain_not_run_file"
  elif (( package_pass_ran )); then
    while IFS=$'\t' read -r plain_where plain_line; do
      [[ -n "$plain_line" ]] && printf '%s\t0\t%s\n' "$plain_where" "$plain_line" >>"$crash_point_replay_plain_not_run_file"
    done < <(test_not_run_lines "$pass_log")
  fi
  crash_point_replay_plain_not_run_total="$(grep -c . "$crash_point_replay_plain_not_run_file")"
  if (( package_pass_ran && plan_ok )); then
    package_pass_report="验证包 $(plan_report "$plan_output" "$crash_point_replay_plain_not_run_total")；跑了的 ${binary_count} 个测试二进制 ${passed_total} 条通过、${ignored_total} 条 ignored（登记的崩溃枚举用例留给按块那一路）"
  elif (( package_pass_ran )); then
    package_pass_report="验证包 ${binary_count} 个测试二进制，${passed_total} 条通过、${ignored_total} 条 ignored、本次未跑 ${crash_point_replay_plain_not_run_total} 处（按条复用算不出，全部现判；登记的崩溃枚举用例留给按块那一路）"
  fi
  if (( plan_ok && package_pass_ran && accounting_ok )); then
    record_green_tests "$kind" "$plan_file" "$plan_output" "$accounting" "$CRASH_CASE_REPLAY_CELL_NAME" \
      "cargo test --release -p singlefs-verification --lib --tests（不带特性那一趟）" || package_pass_plan_red=1
  fi
  echo "  ✓ 验证包里的测试函数跑完（release，登记的崩溃枚举用例留给按块那一路）：${package_pass_report}"
  return 0
}

# package_pass_reuse_held：按条复用失灵了（package_pass_plan_red，plan_failure_verdict 判的）就判红、返回 1；按块那一半照旧先跑完
package_pass_reuse_held() {
  (( ! package_pass_plan_red )) && return 0
  crash_point_replay_unavailable_only "$CRASH_CASE_REPLAY_CELL_NAME" && return 1   # 记库那一步 prune-server 不可用：不可用那一句已经打了，不是失灵
  echo "  ✗ $CRASH_CASE_REPLAY_CELL_NAME 这一格判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；这一趟照旧全部现判跑完、用例都过了"
  echo "     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再只跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME；不能复用就判红，不退回全部现判当绿"
  return 1
}

# crash_case_judged_by_test <键>：这条用例登记了 red-blocks=judged-by-test 没有（admission.py crash-case-judged-by-test）：退 0 登记了、退 1 没登记；
# 判不出（它退别的）退 2、原因放进 judged_by_test_problem。弄坏开关 judged-by-test-unrecognized 下一律答没登记（照没认这个标记，红块照判）。
crash_case_judged_by_test() {
  local answer answer_exit
  [[ "$crash_point_replay_break" == judged-by-test-unrecognized ]] && return 1
  if answer="$(python3 "$crash_point_replay_admission_module" crash-case-judged-by-test "$ROOT" "$1")"; then answer_exit=0; else answer_exit=$?; fi
  if (( answer_exit == 0 || answer_exit == 1 )); then return "$answer_exit"; fi
  judged_by_test_problem="admission.py crash-case-judged-by-test 退 ${answer_exit}：${answer}"
  return 2
}

# run_registered_crash_cases：登记的崩溃枚举用例逐条跑（文件头「用例逐条按块跑」），块按单元键照判定库复用、只判键对不上的块。
# 每条的日志与发现日志放 crash_point_replay_log_directory，跑完不删；全部判绿返回 0，有一条红返回 1（逐条的结论放进 red_cases、green_cases、reused_cases 与块的合计）。
run_registered_crash_cases() {
  local crash_case_row case_key case_package case_target case_function case_label case_log case_findings_file case_run_exit case_findings_exit
  local judged_lines_file judge_output blocks_judged_exit blocks_output blocks_exit blocks_text threads_text started_date finished_date
  local case_threads_origin_text case_threads_note case_way
  local -a blocks_options=()
  red_cases=(); green_cases=(); reused_cases=()
  blocks_reused_total=0; blocks_fresh_total=0; blocks_refused_total=0
  started_date="$(TZ=Asia/Tokyo date +%F)"
  echo "  · 登记的 ${#crash_case_rows[@]} 条崩溃枚举用例逐条跑；块按单元键照判定库（prune-server）复用、只判键对不上的块"
  # 每条用例的挂钟（ITEM_DONE 的秒数，文件头「逐项点名」）：下一条开跑、或全部跑完时记上一条的
  local -A case_seconds=()
  local clock_key="" clock_started=0
  for crash_case_row in "${crash_case_rows[@]}"; do
    IFS=$'\t' read -r case_key case_package case_target case_function <<< "$crash_case_row"
    [[ -z "$clock_key" ]] || case_seconds["$clock_key"]=$((EPOCHSECONDS - clock_started))
    clock_key="$case_key"; clock_started="$EPOCHSECONDS"
    case_label="${case_key#crash-case:}"
    if ! read_crash_case_command "$case_key"; then
      echo "  ✗ $case_key：准入模块交不出起用例的命令：$case_command_problem"
      echo "     → 怎么办：单跑 python3 scripts/gate/admission.py crash-case-command <根> $case_key 看它报什么（取不到 git common-dir、"
      echo "                本机核数读不到、SINGLEFS_CRASH_POINT_REPLAY_THREADS 不是正整数、每台默认重活线程数的登记取不了都在这里报）；修好之后重跑这一道。"
      red_cases+=("$case_key")
      continue
    fi
    case_log="$crash_point_replay_log_directory/log.$case_label"
    case_findings_file="$case_log.findings.tsv"
    rm -f -- "$case_log" "$case_findings_file"
    case_threads_origin_text="没设，取每台默认重活线程数的登记值"
    if [[ "$case_threads_origin" == explicit ]]; then case_threads_origin_text="显式设的"; fi
    case_threads_note="SINGLEFS_CRASH_POINT_REPLAY_THREADS=${case_threads}（${case_threads_origin_text}），本机 ${case_machine_cores} 核"
    case_way="单机跑：${case_command[*]}"
    echo "  · $case_key 开跑（${case_way}；${case_threads_note}）"
    echo "    日志：${case_log}；发现日志：${case_findings_file}"
    if run_crash_case "$case_log" "$case_findings_file" "${case_command[@]}"; then case_run_exit=0; else case_run_exit=$?; fi
    report_pass_outcome "$CRASH_CASE_REPLAY_CELL_NAME" "$case_key 那一趟" "$case_run_exit" "$case_log"
    # 登记了 red-blocks=judged-by-test 的用例（文件头「复用」）：用例的判定是 cargo test 的退出码，块那一行只核在不在、数对不对得上（crash-case-blocks --ran），红块不判；
    # 没登记的块那一行有红判红。弄坏开关 judged-by-test-ignores-exit 下带标记的用例 cargo test 退非 0 也当绿；ran-for-every-case 下每条用例都只核数、不判红块
    if crash_case_judged_by_test "$case_key"; then blocks_judged_exit=0; else blocks_judged_exit=$?; fi
    if (( blocks_judged_exit == 0 )) && [[ "$crash_point_replay_break" == judged-by-test-ignores-exit ]]; then case_run_exit=0; fi
    [[ "$crash_point_replay_break" == ran-for-every-case ]] && blocks_judged_exit=0
    # 发现表跑完就打（判红的那一趟最要看它），判定留到下面
    if report_crash_case_findings "$case_findings_file" "$case_log"; then case_findings_exit=0; else case_findings_exit=$?; fi
    if [[ "$case_run_exit" != 0 ]]; then
      [[ -f "$case_log" ]] && tail -40 "$case_log"
      echo "  ✗ $case_key 判红：cargo test 退非 0（上面是它的尾部）"
      echo "     → 怎么办：单跑看细节：cargo test --release -p $case_package --test $case_target -- --exact $case_function --nocapture"
      echo "                断言消息里是第一处对不上的计数或违例。改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
      red_cases+=("$case_key")
      continue
    fi
    judged_lines_file="$crash_point_replay_scratch_directory/judged.$case_label"
    if ! judge_output="$(python3 "$crash_point_replay_admission_module" crash-case-judge "$ROOT" "$case_key" "$case_log" "$judged_lines_file")"; then
      printf '%s\n' "$judge_output" | sed 's/^/       /'
      echo "  ✗ $case_key 的用例跑过了，日志却判不绿（上面逐条列出）"
      echo "     → 怎么办：计数行不是恰好一行、过滤之后没跑到恰好一条用例，对一对 stage-inputs.tsv 里 $case_key 那一行第三列与用例打印的行；"
      echo "                不是穷举枚举的去 crates/singlefs-verification/src/crash_state_enumeration.rs 的 enumerate_crash_states 看；只起了 1 个线程看 crash_point_replay_worker_threads 读没读到 SINGLEFS_CRASH_POINT_REPLAY_THREADS，"
      echo "                真要单线程跑（比对单进程读数），显式写 SINGLEFS_CRASH_POINT_REPLAY_THREADS=1 bash .claude/gate.d/crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME <根>。"
      red_cases+=("$case_key")
      continue
    fi
    if [[ "$case_findings_exit" != 0 ]]; then
      echo "  ✗ $case_key 的用例跑过了、日志也判得绿，发现日志却判红（上面发现表里的「问题：」逐条列出）"
      echo "     → 怎么办：「这一趟没跑完」看日志 $case_log 的尾部是 panic 还是被杀；red_states 不是 0 是有状态判红而用例没红，"
      echo "                对一对用例的计数断言与 crates/singlefs-verification/src/crash.rs 的发现表；发现日志不在而日志里有 CRASH_POINT_REPLAY_FINDINGS 行，"
      echo "                是 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 没传进用例（看本阶段 run_crash_case）。"
      red_cases+=("$case_key")
      continue
    fi
    blocks_options=()
    if (( blocks_judged_exit == 2 )); then
      echo "  ✗ $case_key：判不出它带没带 red-blocks=judged-by-test（${judged_by_test_problem}），块判定那一行没法定判法"
      echo "     → 怎么办：单跑 python3 scripts/gate/admission.py crash-case-judged-by-test <根> $case_key，照它报的改 .claude/gate.d/stage-inputs.tsv 里那一行"
      red_cases+=("$case_key")
      continue
    fi
    if (( blocks_judged_exit == 0 )); then
      blocks_options=(--ran)
      echo "    $case_key：登记了 red-blocks=judged-by-test，按 cargo test 的退出码判（已退 0），块那一行只核在不在、数对不对得上，红块不判"
    fi
    if blocks_output="$(python3 "$crash_point_replay_admission_module" crash-case-blocks "$ROOT" "$case_log" ${blocks_options[@]+"${blocks_options[@]}"})"; then blocks_exit=0; else blocks_exit=$?; fi
    if (( blocks_exit != 0 )); then
      echo "  ✗ $case_key 的块判定那一行不对（crash-case-blocks${blocks_options[*]:+ ${blocks_options[*]}} 退 $blocks_exit：$blocks_output）"
      echo "     → 怎么办：退 1 是有块判红（没登记 red-blocks=judged-by-test 的用例）：看日志里那几块的发现；退 4 是用例没打恰好一行 CRASH_CASE_BLOCKS、或 blocks 不等于 reused + fresh + refused（接口在 scripts/gate/admission.py 文件头）。"
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
    threads_text="${judge_output//$'\n'/；}${judge_output:+；}${case_threads_note}"
    echo "  ✓ $case_key 判绿（${threads_text}；${blocks_text}），判日志认出的行原样："
    sed 's/^/      /' "$judged_lines_file"
    green_cases+=("$case_key")
  done
  [[ -z "$clock_key" ]] || case_seconds["$clock_key"]=$((EPOCHSECONDS - clock_started))
  # 每条一行 ITEM_DONE（文件头「逐项点名」）：全部块都复用的一件复用、别的一件现判，判红 exit 1。弄坏开关 item-done-unreported 下不打
  if [[ "$crash_point_replay_break" != item-done-unreported ]]; then
    for crash_case_row in "${crash_case_rows[@]}"; do
      case_key="${crash_case_row%%$'\t'*}"
      if crash_point_replay_cell_in "$case_key" ${red_cases[@]+"${red_cases[@]}"}; then
        printf 'ITEM_DONE item=%s seconds=%s judged=1 reused=0 exit=1\n' "$case_key" "${case_seconds[$case_key]:-0}"
      elif crash_point_replay_cell_in "$case_key" ${reused_cases[@]+"${reused_cases[@]}"}; then
        printf 'ITEM_DONE item=%s seconds=%s judged=0 reused=1 exit=0\n' "$case_key" "${case_seconds[$case_key]:-0}"
      else
        printf 'ITEM_DONE item=%s seconds=%s judged=1 reused=0 exit=0\n' "$case_key" "${case_seconds[$case_key]:-0}"
      fi
    done
  fi
  finished_date="$(TZ=Asia/Tokyo date +%F)"
  if (( ${#red_cases[@]} > 0 )); then
    echo "  ✗ ${#red_cases[@]} 条崩溃枚举用例判红：${red_cases[*]}（这一趟判绿 ${#green_cases[@]} 条，其中全部块复用 ${#reused_cases[@]} 条；开跑 ${started_date}，跑完 ${finished_date}）"
    echo "     → 怎么办：逐条照它自己那一句「→ 怎么办」改；改完只重跑红的几条：bash .claude/gate.d/crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME --item <用例名>（判过的块照键复用，只判改到的块）。"
    return 1
  fi
  echo "  ✓ 崩溃状态块：现判 ${blocks_fresh_total}、复用 ${blocks_reused_total}、拒算 ${blocks_refused_total}、本次未跑 0"
  echo "  ✓ 登记的 ${#crash_case_rows[@]} 条崩溃枚举用例这一趟都判绿（${green_cases[*]:-无}），其中全部块复用 ${#reused_cases[@]} 条（${reused_cases[*]:-无}）；开跑 ${started_date}，跑完 ${finished_date}；日志在 ${crash_point_replay_log_directory}/"
  return 0
}
# cpu_judging_lines <日志>：崩溃枚举那一类判段判在 CPU 的行（CRASH_POINT_REPLAY_JUDGE route=cpu，或 CRASH_POINT_REPLAY_PARALLEL_START / _FINISHED 后面带 judge=cpu），
# 一处一行「<它所在的 tests/<名>.rs，或「库与别的二进制」>：<那一行，去掉行首空白>」。记号在行内哪里都认，前面是行首或空白：
# 带 --test-threads 1 的那一趟测试自己打的头一行接在「test <名> ... 」同一行后面（lib_libtest_result_lines.py 的 CPU_JUDGING_FORMS 同样在行内找，这样的记号在那边归到同一行里的那一条测试函数）。
# 弄坏开关 cpu-judging-line-start-only 下照改前只认去掉行首空白后以记号起头的行
cpu_judging_lines() {
  local line_start_only=0
  [[ "$crash_point_replay_break" == cpu-judging-line-start-only ]] && line_start_only=1
  awk -v line_start_only="$line_start_only" '
    /^[[:space:]]*Running / {
      current = "库与别的二进制"
      if (match($0, /tests\/[^\/ ]+\.rs \(/)) current = "tests/" substr($0, RSTART + 6, RLENGTH - 8)
      next
    }
    {
      line = $0
      sub(/^[[:space:]]+/, "", line)
      if (line_start_only == 1) {
        judged = line ~ /^CRASH_POINT_REPLAY_JUDGE route=cpu( |$)/ || (line ~ /^CRASH_POINT_REPLAY_PARALLEL_(START|FINISHED) / && line ~ / judge=cpu( |$)/)
      } else {
        judged = line ~ /(^|[[:space:]])CRASH_POINT_REPLAY_JUDGE route=cpu([[:space:]]|$)/
        if (!judged && match(line, /(^|[[:space:]])CRASH_POINT_REPLAY_PARALLEL_(START|FINISHED) /))
          judged = substr(line, RSTART + RLENGTH - 1) ~ /[[:space:]]judge=cpu([[:space:]]|$)/
      }
      if (judged)
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

# report_feature_items_done <这一格的退出码>：feature-gated 的项逐项打 ITEM_DONE（文件头「逐项点名」），stdout。没跑到起 cargo 那一步（读不出挂特性的文件、计划那一步不可用）的不打。
#   秒数：这一项那几段（cargo 的「Running unittests src/lib.rs」「Running tests/<名>.rs」起到下一段）libtest 汇总行「finished in <秒>s」之和，四舍五入成整数；
#   件数：计划里这一项的目标根文件的行（judge 与 refused 算现判、reuse 算复用）；计划算不出的按日志里这一段的结果行数算现判、复用 0；
#   退出码：这一格判红时这一项那几段有 FAILED 的记 1，一段都认不出红的（编不过、判在 CPU 这一类）点到的每一项都记 1；不红时跑到加复用 0 条、有没卡没跑的 GPU 对拍记 77，别的 0。
#   弄坏开关 item-done-unreported 下一行都不打
report_feature_items_done() {
  local cell_exit="$1" item root counts judged reused seconds failed exit_code any_failed=0
  ((crash_point_replay_feature_done_reached)) || return 0
  [[ "$crash_point_replay_break" == item-done-unreported ]] && return 0
  # 每段一行「目标根文件<TAB>finished in 秒数之和<TAB>FAILED 条数<TAB>结果行条数」
  counts="$(awk '
    /^[[:space:]]*Running / {
      current = ""
      if (match($0, /Running unittests src\/lib\.rs /)) current = "src/lib.rs"
      else if (match($0, /Running tests\/[^\/ ]+\.rs \(/)) current = substr($0, RSTART + 8, RLENGTH - 10)
      next
    }
    current == "" { next }
    /^test .* \.\.\. FAILED$/ { failed[current]++ }
    /^test .* \.\.\. (ok|FAILED)$/ { results[current]++ }
    /^test result: / && match($0, /finished in [0-9.]+s/) { seconds[current] += substr($0, RSTART + 12, RLENGTH - 13); seen[current] = 1 }
    END { for (name in seen) printf "%s\t%s\t%d\t%d\n", name, seconds[name] + 0, failed[name] + 0, results[name] + 0
          for (name in failed) if (!(name in seen)) printf "%s\t0\t%d\t%d\n", name, failed[name], results[name] + 0 }' "$crash_point_replay_feature_done_log" 2>/dev/null)"
  awk -F'\t' '$3 > 0 { found = 1 } END { exit found ? 0 : 1 }' <<<"$counts" && any_failed=1
  for item in "${crash_point_replay_feature_done_items[@]}"; do
    root="tests/${item#"$FEATURE_GATED_ITEM_PREFIX"}"
    [[ "$item" == "$FEATURE_GATED_LIBRARY_ITEM" ]] && root="src/lib.rs"
    seconds="$(awk -F'\t' -v root="$root" '$1 == root { printf "%d", $2 + 0.5 }' <<<"$counts")"
    failed="$(awk -F'\t' -v root="$root" '$1 == root { print $3 }' <<<"$counts")"
    if [[ -n "$crash_point_replay_feature_done_plan" && -f "$crash_point_replay_feature_done_plan" ]]; then
      read -r judged reused < <(awk -F'\t' -v root="$root" 'NF >= 7 && $5 == root { if ($2 == "judge" || $2 == "refused") judge++; else if ($2 == "reuse") reuse++ }
        END { print judge + 0, reuse + 0 }' "$crash_point_replay_feature_done_plan")
    else
      judged="$(awk -F'\t' -v root="$root" '$1 == root { print $4 }' <<<"$counts")"; reused=0
    fi
    exit_code=0
    if ((cell_exit != 0)); then
      if ((${failed:-0} > 0)) || ((any_failed == 0)); then exit_code=1; fi
    elif crash_point_replay_cell_in "$item" ${crash_point_replay_feature_done_not_run[@]+"${crash_point_replay_feature_done_not_run[@]}"}; then
      exit_code=77
    fi
    printf 'ITEM_DONE item=%s seconds=%s judged=%s reused=%s exit=%s\n' "$item" "${seconds:-0}" "${judged:-0}" "${reused:-0}" "$exit_code"
  done
}

# run_feature_gated_tests_cell <crash-case-replay 格那一趟的日志，没跑传空串>：起 run_feature_gated_tests_pass，收尾逐项打 ITEM_DONE（report_feature_items_done）；退出码照那一趟
run_feature_gated_tests_cell() {
  local pass_exit
  crash_point_replay_feature_done_reached=0; crash_point_replay_feature_done_items=(); crash_point_replay_feature_done_not_run=()
  crash_point_replay_feature_done_log=""; crash_point_replay_feature_done_plan=""
  if run_feature_gated_tests_pass "$@"; then pass_exit=0; else pass_exit=$?; fi
  report_feature_items_done "$pass_exit"
  return "$pass_exit"
}

# run_feature_gated_tests_pass <crash-case-replay 格那一趟的日志，没跑传空串>：判红打出路、返回 1；不红返回 0，结论放进 feature_cell_state。
# 按条复用（文件头「复用」）：这一组特性下每条测试函数各一条判定，与不带特性那一趟同一类、同一份库（种类 <阶段文件名>:test-functions），
# 键相同的经 --skip 跳过，一条都不用现判就不起 cargo；不带特性那一趟打了 NOT_RUN_ 的测试函数所在的测试目标一并带上补判（文件头「多机配置开了 GPU」）。
# 挂特性的文件「跑到 0 条」按跑到的加复用的算，同一条不数两遍。
run_feature_gated_tests_pass() {
  local tests_directory="crates/singlefs-verification/tests" listing feature_list feature_log feature_exit=0 bindgen_arguments=""
  local amplification_library="$crash_point_replay_stage_repository/scripts/crash/crash-amplification-lib.sh"
  local counts_file="$crash_point_replay_scratch_directory/feature-gated-counts"
  local row file_name file_scope file_features scope_text count ran_total=0 gpu_where gpu_line not_run_total
  local -a gated_rows=() features=() target_arguments=() bindgen_setting=() cleared_environment=(-u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE) zero_files=() gpu_not_run=() gpu_test_arguments=()
  if ! listing="$(list_feature_gated_test_files "$tests_directory")"; then
    echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：读不出 $tests_directory 下哪几个测试文件挂特性 cfg（上面是 python 的报错）"
    echo "     → 怎么办：看报错里点名的文件（多半不是 UTF-8 或读不了），修好之后只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME"
    return 1
  fi
  if [[ -z "$listing" ]]; then
    feature_cell_state="无对象可判"
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：$tests_directory 下 0 个测试文件挂特性 cfg，无对象可判，带特性那一趟不起（往 GATE_NOT_RUN_FILE 记一行）"
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
      printf '%s\n' "crash-point-replay.sh 的 $FEATURE_GATED_TESTS_CELL_NAME 格无对象可判：$tests_directory 下没有挂特性 cfg 的测试文件" >> "$GATE_NOT_RUN_FILE"
    fi
    return 0
  fi
  mapfile -t gated_rows <<< "$listing"
  # 逐项点名（文件头「逐项点名」）：点了 feature-gated 的项只留点名的测试文件、--lib 只在点了 feature-gated:lib 时带；特性照整格那一组（全部挂特性的文件的并集），
  #   键与整格的相同。弄坏开关 feature-items-ignored 下点了也照整格跑；feature-item-features-narrowed 下特性只取点名的文件的
  local -a library_arguments=(--lib) named_rows=()
  if ((${#crash_point_replay_named_feature_items[@]})) && [[ "$crash_point_replay_break" != feature-items-ignored ]]; then
    for row in "${gated_rows[@]}"; do
      crash_point_replay_cell_in "$FEATURE_GATED_ITEM_PREFIX${row%%$'\t'*}" "${crash_point_replay_named_feature_items[@]}" && named_rows+=("$row")
    done
    crash_point_replay_cell_in "$FEATURE_GATED_LIBRARY_ITEM" "${crash_point_replay_named_feature_items[@]}" || library_arguments=()
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：只编、只跑 --item 点名的 ${#crash_point_replay_named_feature_items[@]} 项：${crash_point_replay_named_feature_items[*]}（$tests_directory 下挂特性的 ${#gated_rows[@]} 个文件与 --lib 共 $(( ${#gated_rows[@]} + 1 )) 项；特性照整格那一组）"
    if [[ "$crash_point_replay_break" == feature-item-features-narrowed ]]; then listing="$(printf '%s\n' ${named_rows[@]+"${named_rows[@]}"})"; fi
    gated_rows=(${named_rows[@]+"${named_rows[@]}"})
  fi
  mapfile -t features < <(cut -f3 <<< "$listing" | tr ',' '\n' | grep . | LC_ALL=C sort -u)
  feature_list="$(IFS=,; printf '%s' "${features[*]}")"
  if [[ "$crash_point_replay_break" == features-not-from-source ]]; then feature_list="sample-store,gpu"; fi
  for row in ${gated_rows[@]+"${gated_rows[@]}"}; do target_arguments+=(--test "${row%%.rs$'\t'*}"); done
  # 这一趟跑到的项（收尾时 report_feature_items_done 逐项打 ITEM_DONE，文件头「逐项点名」）
  crash_point_replay_feature_done_items=()
  ((${#library_arguments[@]})) && crash_point_replay_feature_done_items+=("$FEATURE_GATED_LIBRARY_ITEM")
  for row in ${gated_rows[@]+"${gated_rows[@]}"}; do crash_point_replay_feature_done_items+=("$FEATURE_GATED_ITEM_PREFIX${row%%$'\t'*}"); done
  # 特性里有 gpu：测试线程只开 1 个（GPU 对拍的几个测试各占一张卡的显存，并行起会互相抢；一个测试里 CPU 判器自己的线程不受这个参数管：读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，没设取本机可用并行度）
  if [[ ",$feature_list," == *,gpu,* && "$crash_point_replay_break" != gpu-test-threads-unset ]]; then gpu_test_arguments=(--test-threads 1); fi
  # 特性里有 gpu：这一趟要上卡，卡取 task-server 交的（文件头「显卡与清场」）；没有就判红、这一趟不起
  if [[ ",$feature_list," == *,gpu,* ]]; then
    require_service_card "$FEATURE_GATED_TESTS_CELL_NAME" "带 gpu 特性那一趟" || return 1
  else
    reset_service_card
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
  crash_point_replay_feature_pass_cleared_environment
  cleared_environment=("${crash_point_replay_feature_cleared_environment[@]}")
  crash_point_replay_test_functions_environment=("${cleared_environment[@]}")   # test-functions-request 在交给 cargo 的同一份环境里起（crash_point_replay_test_functions_request 上面那一段）
  feature_log="$crash_point_replay_feature_pass_log"
  : > "$feature_log"
  local feature_kind feature_plan_ok=1 feature_ran=1 feature_name feature_plan_red=0 feature_accounting="$crash_point_replay_scratch_directory/features.accounting"
  feature_kind="$(crash_point_replay_verdict_kind "$crash_point_replay_stage_file_name" "$FEATURE_GATED_TESTS_CELL_NAME")"
  local feature_plan_file="$crash_point_replay_scratch_directory/features.plan" feature_plan_output="$crash_point_replay_scratch_directory/features.plan.out"
  local -a feature_plan_options=() feature_roots=() feature_names=() supplement_sources=() supplement_arguments=()
  local supplement_source supplement_name
  local -a supplement_target=()
  ((${#library_arguments[@]})) && feature_roots+=(src/lib.rs)
  for row in ${gated_rows[@]+"${gated_rows[@]}"}; do feature_roots+=("tests/${row%%$'\t'*}"); done
  # 补判（文件头「多机配置开了 GPU」）：不带特性那一趟打了 NOT_RUN_ 的测试函数所在的测试目标一并带上；库那一段本来就在。弄坏开关 no-supplement 下不带；
  #   点了 feature-gated 的项的那一趟不另加目标（文件头「逐项点名」）
  if [[ -s "$crash_point_replay_plain_not_run_file" && "$crash_point_replay_break" != no-supplement ]] && ((${#crash_point_replay_named_feature_items[@]} == 0)); then
    mapfile -t supplement_sources < <(cut -f1 "$crash_point_replay_plain_not_run_file" | LC_ALL=C sort -u)
  fi
  for supplement_source in ${supplement_sources[@]+"${supplement_sources[@]}"}; do
    case "$supplement_source" in
      src/lib.rs) continue ;;
      tests/*.rs) supplement_name="${supplement_source#tests/}"; supplement_target=(--test "${supplement_name%.rs}") ;;
      src/bin/*/main.rs) supplement_name="${supplement_source#src/bin/}"; supplement_target=(--bin "${supplement_name%/main.rs}") ;;
      src/bin/*.rs) supplement_name="${supplement_source#src/bin/}"; supplement_target=(--bin "${supplement_name%.rs}") ;;
      *) continue ;;
    esac
    if [[ " ${target_arguments[*]} " == *" ${supplement_target[*]} "* ]]; then continue; fi
    target_arguments+=("${supplement_target[@]}")
    supplement_arguments+=("${supplement_target[@]}")
    feature_roots+=("$supplement_source")
  done
  if (( ${#supplement_arguments[@]} > 0 )); then
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：不带特性那一趟打了 NOT_RUN_ 的测试函数由这一趟补判，另带上它们所在的测试目标：${supplement_arguments[*]}"
  fi
  IFS=, read -r -a feature_names <<< "$feature_list"
  for feature_name in "${feature_names[@]}"; do feature_plan_options+=(--feature "singlefs-verification/$feature_name"); done
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
    skip_arguments_of_plan -
  fi
  crash_point_replay_feature_plan_ok="$feature_plan_ok"   # 出口 4 那几格读这一趟的行时要（文件头「出口 4 那六格」）
  # 计划那一步 prune-server 不可用（plan_failure_verdict 记了这一格）：不起 cargo，这一格就此收尾
  crash_point_replay_unavailable_only "$FEATURE_GATED_TESTS_CELL_NAME" && return 1
  # 放开 --test-threads 1 的 GPU 测试目标（GPU_TEST_THREADS_RELEASED_TARGETS；文件头「feature-gated-tests 格」）：特性里有 gpu 时从这一趟拿出来另起一趟，
  #   测试线程取每台默认重活线程数（scripts/hosts/heavy-threads.sh 的 heavy_thread_default；它的测试按显存额度排队）；别的目标照旧一趟、带 --test-threads 1。
  #   弄坏开关 gpu-agreement-test-threads-one 下照改前不拿出来（一趟、一律带 1）
  local -a main_target_arguments=() released_target_arguments=() released_test_arguments=()
  local released_threads="" released_log="$feature_log.released" feature_main_log="$feature_log" released_exit=0 argument_index
  for (( argument_index = 0; argument_index < ${#target_arguments[@]}; argument_index += 2 )); do
    if [[ ",$feature_list," == *,gpu,* && "${target_arguments[argument_index]}" == --test && " ${GPU_TEST_THREADS_RELEASED_TARGETS[*]} " == *" ${target_arguments[argument_index + 1]} "* ]] \
       && [[ "$crash_point_replay_break" != gpu-agreement-test-threads-one ]]; then
      released_target_arguments+=("${target_arguments[@]:argument_index:2}")
    else
      main_target_arguments+=("${target_arguments[@]:argument_index:2}")
    fi
  done
  if (( ${#released_target_arguments[@]} > 0 )); then
    released_threads="$( (source "$crash_point_replay_stage_repository/scripts/hosts/heavy-threads.sh" && heavy_thread_default) 2>/dev/null)"
    if [[ ! "$released_threads" =~ ^[1-9][0-9]*$ ]]; then
      echo "  ! $FEATURE_GATED_TESTS_CELL_NAME：取不出每台默认重活线程数（scripts/hosts/heavy-threads.sh 的 heavy_thread_default），${released_target_arguments[*]} 照旧带 --test-threads 1"
      released_threads=1
    fi
    released_test_arguments=(--test-threads "$released_threads")
    feature_main_log="$feature_log.main"
  fi
  echo "── $FEATURE_GATED_TESTS_CELL_NAME：带特性那一趟（特性从 $tests_directory 下 ${#gated_rows[@]} 个挂特性的测试文件现读）：cargo test --release -p singlefs-verification --features $feature_list${library_arguments[*]:+ ${library_arguments[*]}} ${main_target_arguments[*]} -- --nocapture${gpu_test_arguments[*]:+ ${gpu_test_arguments[*]}}"
  if [[ "$crash_point_replay_break" == no-feature-pass ]]; then
    : > "$feature_log"
    feature_main_log="$feature_log"; released_target_arguments=()
  elif (( ! feature_ran )); then
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：这一组特性下要跑的测试函数都有键相同的判定，全部复用，这一趟没起 cargo"
  else
    # 特性里没有 gpu 时按测试目标并行起，有 gpu 的照旧一条 cargo test（文件头「按条复用两趟按测试目标并行起」）
    local -a feature_runner=()
    crash_point_replay_test_runner feature_runner "$([[ ",$feature_list," == *,gpu,* ]] && echo gpu)"
    env "${cleared_environment[@]}" ${crash_point_replay_card_unset[@]+"${crash_point_replay_card_unset[@]}"} ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
      ${crash_point_replay_card_quiesce_environment[@]+"${crash_point_replay_card_quiesce_environment[@]}"} ${bindgen_setting[@]+"${bindgen_setting[@]}"} ${crash_point_replay_card_setting[@]+"${crash_point_replay_card_setting[@]}"} \
      ${crash_point_replay_card_quiesce_prefix[@]+"${crash_point_replay_card_quiesce_prefix[@]}"} "${feature_runner[@]}" --release --no-fail-fast -p singlefs-verification --features "$feature_list" ${library_arguments[@]+"${library_arguments[@]}"} ${main_target_arguments[@]+"${main_target_arguments[@]}"} -- --nocapture ${gpu_test_arguments[@]+"${gpu_test_arguments[@]}"} ${plan_skip_arguments[@]+"${plan_skip_arguments[@]}"} 2>&1 \
      | tee "$feature_log" \
      | { grep --line-buffered '^CRASH_POINT_REPLAY_PROGRESS ' || true; } \
      | sed -u 's/^/    /'
    feature_exit="${PIPESTATUS[0]}"
    # 每一趟当场写结局（文件头「每一趟当场写结局」）：放开的那一趟另起之前先打这一趟的，那一趟撞顶被杀时这一趟的结局已在外层日志里
    if (( ${#released_target_arguments[@]} > 0 )); then cp -- "$feature_log" "$feature_main_log"; fi
    report_pass_outcome "$FEATURE_GATED_TESTS_CELL_NAME" "带特性那一趟（--features $feature_list）" "$feature_exit" "$feature_main_log"
    if (( ${#released_target_arguments[@]} > 0 )); then
      echo "── $FEATURE_GATED_TESTS_CELL_NAME：放开 --test-threads 1 的 GPU 测试目标另起一趟（它的测试按显存额度排队，测试线程取每台默认重活线程数（scripts/hosts/heavy-threads.sh 的 heavy_thread_default）：$released_threads）：cargo test --release -p singlefs-verification --features $feature_list ${released_target_arguments[*]} -- --nocapture --test-threads $released_threads"
      env "${cleared_environment[@]}" ${crash_point_replay_card_unset[@]+"${crash_point_replay_card_unset[@]}"} ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
        ${crash_point_replay_card_quiesce_environment[@]+"${crash_point_replay_card_quiesce_environment[@]}"} ${bindgen_setting[@]+"${bindgen_setting[@]}"} ${crash_point_replay_card_setting[@]+"${crash_point_replay_card_setting[@]}"} \
        ${crash_point_replay_card_quiesce_prefix[@]+"${crash_point_replay_card_quiesce_prefix[@]}"} "${feature_runner[@]}" --release --no-fail-fast -p singlefs-verification --features "$feature_list" "${released_target_arguments[@]}" -- --nocapture "${released_test_arguments[@]}" ${plan_skip_arguments[@]+"${plan_skip_arguments[@]}"} 2>&1 \
        | tee "$released_log" \
        | { grep --line-buffered '^CRASH_POINT_REPLAY_PROGRESS ' || true; } \
        | sed -u 's/^/    /'
      released_exit="${PIPESTATUS[0]}"
      report_pass_outcome "$FEATURE_GATED_TESTS_CELL_NAME" "放开 --test-threads 1 的那一趟（${released_target_arguments[*]}）" "$released_exit" "$released_log"
      (( feature_exit != 0 )) || feature_exit="$released_exit"
      # 两趟的输出并成一份（计数、没卡没跑、判在 CPU、红的目标都照它读）；逐条认结局分开认（这一趟不是 --test-threads 1，记号不按正在跑的那一条认）
      cat -- "$released_log" >> "$feature_log"
    fi
  fi
  # 跑到了这一步（起过 cargo，或全部复用没起）：收尾逐项打 ITEM_DONE（report_feature_items_done 读这几个全局）
  crash_point_replay_feature_done_reached=1
  crash_point_replay_feature_done_log="$feature_log"
  crash_point_replay_feature_done_plan="$([[ "$feature_plan_ok" == 1 ]] && printf '%s' "$feature_plan_output.kept")"
  # 逐条认结局的入参（文件头「复用」；特性里有 gpu 时这一趟带 --test-threads 1，记号认作正在跑的那一条；放开的那一趟另认）：判红、判绿两条路同一份
  local feature_accounting_ok=0 feature_serial=0 released_log_to_account=-
  local -a feature_red_accounting=()
  (( ${#gpu_test_arguments[@]} > 0 )) && feature_serial=1
  (( ${#released_target_arguments[@]} > 0 )) && released_log_to_account="$released_log"
  : >"$feature_accounting"
  if [[ "$feature_exit" != 0 && "$crash_point_replay_break" != ignore-feature-pass-exit ]]; then
    tail -40 "$feature_log"
    grep -E '^test .* FAILED$|^error: test failed, to rerun pass' "$feature_log" | sed 's/^/    /' || true
    (( feature_plan_ok )) && failed_tests_changed_units "$feature_plan_output.kept" "$feature_log"
    report_red_test_targets "$feature_log" "$FEATURE_GATED_TESTS_CELL_NAME" "$crash_point_replay_scratch_directory/red-rows.$FEATURE_GATED_TESTS_CELL_NAME"
    # 同一趟判绿的照键记绿（文件头「复用」）：弄坏开关 feature-pass-red-withholds-green 下照改前不认、不记
    if (( feature_plan_ok )) && [[ "$crash_point_replay_break" != feature-pass-red-withholds-green ]] \
         && account_feature_pass "$feature_main_log" "$feature_serial" "$released_log_to_account" "$feature_accounting"; then
      feature_red_accounting=("$feature_accounting")
    fi
    (( feature_plan_ok )) && record_red_tests "$feature_kind" "$feature_plan_file" "$feature_plan_output" "$crash_point_replay_scratch_directory/red-rows.$FEATURE_GATED_TESTS_CELL_NAME" \
      "$FEATURE_GATED_TESTS_CELL_NAME" "cargo test --release -p singlefs-verification --features $feature_list（带特性那一趟）" ${feature_red_accounting[@]+"${feature_red_accounting[@]}"}
    echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：带特性那一趟 cargo test 退 $feature_exit（上面是它的尾部与全部红的用例、测试目标）"
    echo "     → 怎么办：单跑看细节：BINDGEN_EXTRA_CLANG_ARGS=\"$bindgen_arguments\" cargo test --release -p singlefs-verification --features $feature_list${library_arguments[*]:+ ${library_arguments[*]}} ${target_arguments[*]} -- --nocapture${gpu_test_arguments[*]:+ ${gpu_test_arguments[*]}}；"
    echo "                编不过多半是 librocksdb-sys 现编 C++ 缺 libclang、C++ 编译器或这个变量；用例红照断言消息改；没卡的机器上 GPU 对拍该打一行 NOT_RUN_NO_GPU_ADAPTER 之后返回成功，panic 就是那条用例没接这个约定。"
    return 1
  fi
  # 配置开了 GPU、这一趟编进了 gpu（文件头「多机配置开了 GPU」）：判段必须在 GPU，日志里有判在 CPU 的行判红，同一趟的判定照记
  if [[ ",$feature_list," == *,gpu,* ]] && (( crash_point_replay_gpu_configured )) && [[ "$crash_point_replay_break" != cpu-judging-accepted ]]; then
    local feature_cpu_judging=()
    mapfile -t feature_cpu_judging < <(cpu_judging_lines "$feature_log")
    if (( ${#feature_cpu_judging[@]} > 0 )); then
      printf '      %s\n' "${feature_cpu_judging[@]:0:20}"
      # 判在 CPU 归得到的那几条记红、归不到的整个二进制不记、别的判绿的照键记绿（record_cpu_judging_pass；逐条认结局的入参同判红那一段）：
      # 弄坏开关 feature-pass-cpu-judging-records-nothing 下照改前一条都不记
      if (( feature_plan_ok && feature_ran )) && [[ "$crash_point_replay_break" != feature-pass-cpu-judging-records-nothing ]] \
           && account_feature_pass "$feature_main_log" "$feature_serial" "$released_log_to_account" "$feature_accounting"; then
        record_cpu_judging_pass "$feature_kind" "$feature_plan_file" "$feature_plan_output" "$feature_accounting" "$FEATURE_GATED_TESTS_CELL_NAME" \
          "cargo test --release -p singlefs-verification --features $feature_list（带特性那一趟）"
      fi
      echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：带 gpu 特性那一趟判在 CPU：多机配置 $crash_point_replay_configuration_file 写了 ENABLE_GPU=1、这一趟编进了 gpu，日志里有 ${#feature_cpu_judging[@]} 处判段判在 CPU（上面列了前 20 处）"
      echo "     → 怎么办：看那几行的 why_not_the_gpu=（选路在 crates/singlefs-verification/src/crash_state_enumeration.rs 的 crash_point_replay_judge_route）：用例读的配置与这里判的不是同一份，就对齐 SINGLEFS_MULTI_HOST_CONFIG；改完只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME"
      return 1
    fi
  fi
  count_tests_run_by_file "$feature_log" ${1:+"$1"} > "$counts_file"
  # 键相同而复用的也算跑到（这一组特性下的计划，与不带特性那一趟的计划）：一条一行「文件名<TAB>条数」。两趟一份库，同一条测试不数两遍：
  # 这一趟现判（跑了、日志里数过）的，与不带特性那一趟现判、日志也数进来了的，不再按复用数
  local -a reused_plan_files=("$feature_plan_output.kept")
  [[ -f "$crash_point_replay_scratch_directory/plain.plan.out" ]] && reused_plan_files+=("$crash_point_replay_scratch_directory/plain.plan.out")
  awk -F'\t' -v kept="$feature_plan_output.kept" -v plain_counted="${1:+1}" '
    NF >= 7 && $5 ~ /^tests\// {
      if ($2 == "reuse") reuse[$1] = $5
      else if (($2 == "judge" || $2 == "refused") && (FILENAME == kept || plain_counted == 1)) ran[$1] = 1
    }
    END { for (name in reuse) if (!(name in ran)) { source = reuse[name]; sub(/^tests\//, "", source); count[source]++ } for (source in count) printf "%s\t%d\n", source, count[source] }' \
    "${reused_plan_files[@]}" > "$counts_file.reused"
  for row in "${gated_rows[@]}"; do
    IFS=$'\t' read -r file_name file_scope file_features <<< "$row"
    count="$(awk -F'\t' -v name="$file_name" '$1 == name { print $2 }' "$counts_file")"
    count="${count:-0}"
    count=$((count + $(awk -F'\t' -v name="$file_name" '$1 == name { total += $2 } END { print total + 0 }' "$counts_file.reused")))
    ran_total=$((ran_total + count))
    if [[ "$count" == 0 && "$crash_point_replay_break" != ignore-zero-feature-files ]]; then
      zero_files+=("tests/$file_name")
      crash_point_replay_feature_done_not_run+=("$FEATURE_GATED_ITEM_PREFIX$file_name")
      scope_text="逐条挂"; if [[ "$file_scope" == whole-file ]]; then scope_text="整份挂"; fi
      crash_point_replay_report_not_run "验证包 $tests_directory/$file_name ${scope_text}特性 $file_features，跑到它的用例 0 条（cargo 输出里没有它那一段，或只有 ignored）：给它补一条不标 ignored 的用例，或查 cfg 里的特性名与 Cargo.toml 的 [features] 对不对得上"
    fi
  done
  if [[ "$crash_point_replay_break" != ignore-gpu-not-run ]]; then
    while IFS=$'\t' read -r gpu_where gpu_line; do
      [[ -n "$gpu_line" ]] || continue
      gpu_not_run+=("$gpu_line")
      if [[ "$gpu_where" == tests/* ]]; then crash_point_replay_feature_done_not_run+=("$FEATURE_GATED_ITEM_PREFIX${gpu_where#tests/}")
      else crash_point_replay_feature_done_not_run+=("$FEATURE_GATED_LIBRARY_ITEM"); fi
      crash_point_replay_report_not_run "GPU 对拍没跑（${gpu_where}；这台机器找不到 GPU 适配器）：${gpu_line}；在有卡的机器上跑这一格补上"
    done < <(gpu_adapter_not_run_lines "$feature_log")
  fi
  # 逐条认结局与记号（account_feature_pass，入参在判红那一段之前定好），判绿的照键记库
  if (( feature_ran )) && [[ "$crash_point_replay_break" != no-feature-pass ]] \
       && account_feature_pass "$feature_main_log" "$feature_serial" "$released_log_to_account" "$feature_accounting"; then
    feature_accounting_ok=1
  fi
  if (( feature_plan_ok && feature_ran && feature_accounting_ok )); then
    record_green_tests "$feature_kind" "$feature_plan_file" "$feature_plan_output" "$feature_accounting" "$FEATURE_GATED_TESTS_CELL_NAME" \
      "cargo test --release -p singlefs-verification --features $feature_list（带特性那一趟）" || feature_plan_red=1
  fi
  supplement_plain_not_run "$feature_plan_output" "$feature_accounting" "$feature_plan_ok"
  crash_point_replay_unavailable_only "$FEATURE_GATED_TESTS_CELL_NAME" && return 1   # 记库那一步 prune-server 不可用：不可用那一句已经打了，不是失灵
  if (( feature_plan_red )); then
    echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME 这一格判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；带特性那一趟照旧全部现判跑完了"
    echo "     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再只跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME；不能复用就判红，不退回全部现判当绿"
    return 1
  fi
  not_run_total=$(( ${#zero_files[@]} + ${#gpu_not_run[@]} ))
  if (( not_run_total > 0 )); then
    feature_cell_state="有 ${not_run_total} 处本次未跑"
    echo "  ! $FEATURE_GATED_TESTS_CELL_NAME 不算全绿：${#gated_rows[@]} 个挂特性的测试文件（特性 $feature_list）跑到 ${ran_total} 条，本次未跑 ${not_run_total} 处：编进来 0 条的文件 ${#zero_files[@]} 个（${zero_files[*]:-无}），没卡没跑的 GPU 对拍 ${#gpu_not_run[@]} 条"
    echo "     → 编进来 0 条的给它补不标 ignored 的用例；GPU 对拍在有卡的机器上补跑：bash .claude/gate.d/crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME"
  else
    feature_cell_state="绿"
    echo "  ✓ $FEATURE_GATED_TESTS_CELL_NAME：${#gated_rows[@]} 个挂特性的测试文件（特性 $feature_list）每个都编进来、跑到了用例，共 ${ran_total} 条（含复用；$( (( feature_plan_ok )) && printf '按条：%s' "$(plan_report "$feature_plan_output.kept" 0)" || printf '按条复用算不出，全部现判')）"
  fi
  return 0
}

# ── 产品 GPU 包带 tests-on-a-real-gpu 的真显卡对拍（product-gpu-real-card-tests 格；文件头「product-gpu-real-card-tests 格」）─────────
# product_gpu_target_rows <包目录> <特性>：这个包里开了这个特性才编的测试目标，一个一行「目标名<TAB>源文件（包目录起）」，照目标名排序：Cargo.toml 的 [[test]] 里
# required-features 有这个特性的（源文件取它的 path，没写取 tests/<名>.rs），加 tests/ 顶层每个 .rs（// 起头的行不读）里 cfg 写了 feature = "<特性>" 的。按包与特性认，不按文件名写死。
# 退出码：0 有（逐行打）、3 包不在（没有 Cargo.toml）、4 包的 [features] 里没有这个特性、5 包有这个特性而没有测试目标挂它、6 读不了（原因打在标准错误）。
product_gpu_target_rows() {
  python3 - "$1" "$2" <<'PYTHON_PRODUCT_GPU_TARGETS'
import pathlib
import re
import sys
import tomllib

package_directory, feature = pathlib.Path(sys.argv[1]), sys.argv[2]
manifest = package_directory / "Cargo.toml"
if not manifest.is_file():
    sys.exit(3)
try:
    data = tomllib.loads(manifest.read_text(encoding="utf-8"))
    if feature not in (data.get("features") or {}):
        sys.exit(4)
    targets = {}
    for entry in data.get("test") or []:
        if isinstance(entry, dict) and entry.get("name") and feature in (entry.get("required-features") or []):
            targets[entry["name"]] = entry.get("path") or f"tests/{entry['name']}.rs"
    attribute = re.compile(r'\bfeature\s*=\s*"' + re.escape(feature) + '"')
    for path in sorted((package_directory / "tests").glob("*.rs")):
        code = "\n".join(line for line in path.read_text(encoding="utf-8").splitlines() if not line.lstrip().startswith("//"))
        if attribute.search(code):
            targets.setdefault(path.stem, f"tests/{path.name}")
except (OSError, ValueError) as error:
    print(f"{type(error).__name__}: {error}", file=sys.stderr)
    sys.exit(6)
if not targets:
    sys.exit(5)
for name, source in sorted(targets.items()):
    print(f"{name}\t{source}")
PYTHON_PRODUCT_GPU_TARGETS
}

# case_command_uses_card：read_crash_case_command 交的 case_command 带 gpu 特性（--features 的值里有 gpu）就是要上卡的用例
case_command_uses_card() {
  local index
  for (( index = 0; index + 1 < ${#case_command[@]}; index++ )); do
    [[ "${case_command[index]}" == --features && ",${case_command[index + 1]}," == *,gpu,* ]] && return 0
  done
  return 1
}

# reset_service_card：这一趟不上卡，清掉 require_service_card 放进的几个全局数组
reset_service_card() {
  crash_point_replay_card_setting=() crash_point_replay_card_unset=() crash_point_replay_card_quiesce_prefix=() crash_point_replay_card_quiesce_environment=()
}

# require_service_card <格名> <哪一趟>：要上卡的一趟取 task-server 交的显卡（文件头「显卡与清场」）。环境里有 GPU_CARDS_BY_PRIORITY 就放进全局数组
#   crash_point_replay_card_setting（交给 cargo 的 env 赋值）、打一行用的是哪张卡，返回 0；没有就打 ✗ 返回 1（这一趟不起）。
#   弄坏开关 card-missing-unrefused：没有也照起；service-card-not-forwarded：有也不交，env 里另 -u 掉（crash_point_replay_card_unset）；
#   stage-quiesces-by-itself：照改前在这一趟前面套 scripts/crash/gpu-quiesce-run.sh（crash_point_replay_card_quiesce_prefix；清场配置写成草稿目录里只写假清场命令的一份，
#   状态目录与日志目录也在草稿目录里：开关只在样本根上认）
require_service_card() {
  reset_service_card
  local card="${GPU_CARDS_BY_PRIORITY:-}"
  if [[ -z "$card" && "$crash_point_replay_break" != card-missing-unrefused ]]; then
    echo "  ✗ $1：$2要上卡，环境里却没有 task-server 交的显卡（GPU_CARDS_BY_PRIORITY），这一趟没起"
    echo "     → 怎么办：这道门禁经 task-server/task submit -- bash .claude/gate.d/crash-point-replay.sh … 交（服务认出它要卡、套 scripts/crash/gpu-quiesce-run.sh 清场、经 GPU_CARDS_BY_PRIORITY 交卡），不直接起；样本在 .crash-point-replay-sample-tools/caller-environment 里写一行 GPU_CARDS_BY_PRIORITY=<HOSTS 里的地址>:<卡号>:<额度>"
    return 1
  fi
  if [[ "$crash_point_replay_break" == service-card-not-forwarded ]]; then
    crash_point_replay_card_unset=(-u GPU_CARDS_BY_PRIORITY)
  elif [[ -n "$card" ]]; then
    crash_point_replay_card_setting=("GPU_CARDS_BY_PRIORITY=$card")
  fi
  if [[ "$crash_point_replay_break" == stage-quiesces-by-itself ]]; then
    printf '%s\n' QUIESCE_STOP_COMMAND=true QUIESCE_STOPPED_CHECK_COMMAND=true > "$crash_point_replay_scratch_directory/stage-quiesce.env"
    crash_point_replay_card_quiesce_prefix=(bash "$crash_point_replay_stage_repository/scripts/crash/gpu-quiesce-run.sh" --)
    crash_point_replay_card_quiesce_environment=("SINGLEFS_MULTI_HOST_CONFIG=$crash_point_replay_scratch_directory/stage-quiesce.env"
      "GPU_QUIESCE_STATE_DIRECTORY=$crash_point_replay_scratch_directory/gpu-quiesce-state" "GPU_QUIESCE_LOG_DIRECTORY=$crash_point_replay_scratch_directory/gpu-quiesce-logs")
  fi
  echo "  · $1：$2要上卡，显卡是 task-server 交的 ${card:-（没有）}（清场归 task-server，这里不另套清场包装）"
}

# run_product_gpu_real_card_cell：判红打出路、返回 1；不红返回 0，结论放进 product_gpu_cell_state。
# 按条复用同 run_feature_gated_tests_cell（同一类判定库 <阶段文件名>:test-functions，键相同的经 --skip 跳过，一条都不用现判就不起 cargo）；
# 只跑对拍目标（--test <名>…，不带 --lib：库单测是不带特性那一趟的事，也不上卡），上 task-server 交的卡、--test-threads 1。
run_product_gpu_real_card_cell() {
  local package="$PRODUCT_GPU_PACKAGE_NAME" feature="$PRODUCT_GPU_FEATURE_NAME" cell="$PRODUCT_GPU_REAL_CARD_CELL_NAME"
  local package_directory="crates/$PRODUCT_GPU_PACKAGE_NAME" scratch="$crash_point_replay_scratch_directory"
  local log="$crash_point_replay_scratch_directory/$PRODUCT_GPU_PACKAGE_NAME-real-card.log" accounting="$crash_point_replay_scratch_directory/product-gpu.accounting"
  local plan_file="$crash_point_replay_scratch_directory/product-gpu.plan" plan_output="$crash_point_replay_scratch_directory/product-gpu.plan.out"
  local listing listing_exit=0 row kind cell_exit=0 plan_ok=1 plan_red=0 ran=1 ran_total=0 count file_name not_run_total gpu_where gpu_line
  local plan_package="$PRODUCT_GPU_PACKAGE_NAME"   # plan_tests 与 seed_prior_verdicts 读它：算产品 GPU 包的键
  local -a target_rows=() target_arguments=() zero_files=() gpu_not_run=() gpu_test_arguments=(--test-threads 1) cleared_environment=(-u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE)
  local -a feature_roots=()
  if listing="$(product_gpu_target_rows "$package_directory" "$feature" 2>"$scratch/product-gpu.targets.err")"; then
    :
  else
    listing_exit=$?
  fi
  if (( listing_exit == 0 )) && [[ "$crash_point_replay_break" == product-gpu-targets-by-file-name ]]; then
    # 弄坏开关：对拍目标按文件名认（只认 tests/checksum 起头的，改前按文件名写死的样子），不按包与特性认
    listing="$(awk -F'\t' '$2 ~ /^tests\/checksum/' <<< "$listing")"
    [[ -n "$listing" ]] || listing_exit=5
  fi
  case "$listing_exit" in
    0) ;;
    3|5)
      product_gpu_cell_state="无对象可判"
      echo "  · $cell：$( ((listing_exit == 3)) && echo "$package_directory/Cargo.toml 不在（没有产品 GPU 包）" || echo "$package 的 tests/ 与 [[test]] 里没有测试目标挂 $feature" )，无对象可判，这一趟不起（往 GATE_NOT_RUN_FILE 记一行）"
      if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
        printf '%s\n' "crash-point-replay.sh 的 $cell 格无对象可判：$package 包里没有开 $feature 才编的测试目标" >> "$GATE_NOT_RUN_FILE"
      fi
      return 0 ;;
    4)
      echo "  ✗ $cell：$package_directory/Cargo.toml 的 [features] 里没有 $feature：这一格认的特性名对不上包了（改名或删了而这一格还认旧名，真显卡对拍就没有门禁在跑）"
      echo "     → 怎么办：包里改了特性名，就把这一道的 PRODUCT_GPU_FEATURE_NAME 与 crates-mutation-rows.py 的 GPU_TEST_FEATURES 一起改成新名；"
      echo "                对拍测试整个撤了，就把这一格与那几处一起撤；改完只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $cell"
      return 1 ;;
    *)
      echo "  ✗ $cell：读不出 $package_directory 里哪几个测试目标挂 $feature（$(tail -n 1 "$scratch/product-gpu.targets.err" 2>/dev/null)）"
      echo "     → 怎么办：看上面那句里点名的文件（多半 Cargo.toml 写坏了或测试文件不是 UTF-8），修好之后只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $cell"
      return 1 ;;
  esac
  mapfile -t target_rows <<< "$listing"
  for row in "${target_rows[@]}"; do
    target_arguments+=(--test "${row%%$'\t'*}")
    feature_roots+=("${row#*$'\t'}")
  done
  [[ "$crash_point_replay_break" == product-gpu-test-threads-unset ]] && gpu_test_arguments=()
  require_service_card "$cell" "真显卡对拍那一趟" || return 1
  crash_point_replay_feature_pass_cleared_environment
  cleared_environment=("${crash_point_replay_feature_cleared_environment[@]}")
  crash_point_replay_test_functions_environment=("${cleared_environment[@]}")   # 同 run_feature_gated_tests_cell：test-functions-request 在交给 cargo 的同一份环境里起
  : > "$log"
  # 按条复用（同 run_feature_gated_tests_cell）：这一组特性下对拍目标里每条测试函数各一条判定
  kind="$(crash_point_replay_verdict_kind "$crash_point_replay_stage_file_name" "$cell")"
  plan_skip_arguments=()
  if plan_tests "$kind" "$plan_file" "$plan_output" --feature "$package/$feature"; then
    awk -F'\t' -v roots="$(IFS=' '; printf '%s' "${feature_roots[*]}")" 'BEGIN { n = split(roots, list, " "); for (i = 1; i <= n; i++) keep[list[i]] = 1 }
      NF >= 7 && ($5 in keep) { print }' "$plan_output" > "$plan_output.kept"
    plan_judge="$(awk -F'\t' '$2 == "judge"' "$plan_output.kept" | grep -c .)"; plan_reuse="$(awk -F'\t' '$2 == "reuse"' "$plan_output.kept" | grep -c .)"
    plan_refused="$(awk -F'\t' '$2 == "refused"' "$plan_output.kept" | grep -c .)"
    skip_arguments_of_plan "$plan_output.kept"
    echo "  · $cell 按条复用：$package 的 ${#target_rows[@]} 个对拍测试目标里 $((plan_judge + plan_reuse + plan_refused)) 条编得进的测试函数，$(plan_report "$plan_output.kept")"
    plan_listing "$plan_output.kept"
    (( plan_judge + plan_refused == 0 )) && ran=0
  else
    plan_ok=0
    : > "$plan_output.kept"
    echo "  ! $cell 按条复用算不出（$plan_problem）：这一趟全部现判、一条都不记库"
    ((${#plan_failure[@]})) && { plan_failure_verdict "$cell" "${plan_failure[@]}" || plan_red=1; }
  fi
  # 计划那一步 prune-server 不可用（plan_failure_verdict 记了这一格）：不起 cargo，这一格就此收尾
  crash_point_replay_unavailable_only "$cell" && return 1
  echo "── $cell：cargo test --release -p $package --features $feature ${target_arguments[*]} -- --nocapture${gpu_test_arguments[*]:+ ${gpu_test_arguments[*]}}"
  if (( ! ran )); then
    echo "  · $cell：这一组特性下要跑的测试函数都有键相同的判定，全部复用，这一趟没起 cargo"
  elif [[ "$crash_point_replay_break" == product-gpu-cell-skipped ]]; then
    : > "$log"   # 弄坏开关：要现判的也不起对拍那一趟（日志空着、退出码当 0）
  else
    env "${cleared_environment[@]}" ${crash_point_replay_card_unset[@]+"${crash_point_replay_card_unset[@]}"} ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
      ${crash_point_replay_card_quiesce_environment[@]+"${crash_point_replay_card_quiesce_environment[@]}"} ${crash_point_replay_card_setting[@]+"${crash_point_replay_card_setting[@]}"} \
      ${crash_point_replay_card_quiesce_prefix[@]+"${crash_point_replay_card_quiesce_prefix[@]}"} \
      cargo test --release --no-fail-fast -p "$package" --features "$feature" "${target_arguments[@]}" -- --nocapture ${gpu_test_arguments[@]+"${gpu_test_arguments[@]}"} \
      ${plan_skip_arguments[@]+"${plan_skip_arguments[@]}"} 2>&1 \
      | tee "$log" \
      | { grep --line-buffered '^CRASH_POINT_REPLAY_PROGRESS ' || true; } \
      | sed -u 's/^/    /'
    cell_exit="${PIPESTATUS[0]}"
  fi
  # 逐条认结局的入参（文件头「复用」；这一趟带 --test-threads 1，记号认作正在跑的那一条）：判红、判绿两条路同一份
  local accounting_ok=0 serial=0
  local -a red_accounting=()
  (( ${#gpu_test_arguments[@]} > 0 )) && serial=1
  : > "$accounting"
  if [[ "$cell_exit" != 0 && "$crash_point_replay_break" != product-gpu-exit-ignored ]]; then
    tail -40 "$log"
    grep -E '^test .* FAILED$|^error: test failed, to rerun pass' "$log" | sed 's/^/    /' || true
    (( plan_ok )) && failed_tests_changed_units "$plan_output.kept" "$log"
    report_red_test_targets "$log" "$cell" "$scratch/red-rows.$cell"
    # 同一趟判绿的照键记绿（文件头「复用」）：弄坏开关 product-gpu-red-withholds-green 下照改前不认、不记
    if (( plan_ok )) && [[ "$crash_point_replay_break" != product-gpu-red-withholds-green ]] && account_test_results "$log" "$serial" "$accounting"; then
      red_accounting=("$accounting")
    fi
    (( plan_ok )) && record_red_tests "$kind" "$plan_file" "$plan_output" "$scratch/red-rows.$cell" "$cell" "cargo test --release -p $package --features $feature（真显卡对拍）" \
      ${red_accounting[@]+"${red_accounting[@]}"}
    echo "  ✗ $cell：真显卡对拍那一趟 cargo test 退 $cell_exit（上面是它的尾部与全部红的用例、测试目标）：有卡的机器上对拍判红，GPU 核与 CPU 参照对不上"
    echo "     → 怎么办：只重跑这一格看细节：task-server/task submit -- bash .claude/gate.d/crash-point-replay.sh --check $cell（服务认出这道门禁要卡、清场、交卡）；"
    echo "                断言消息里点名对不上的长度与起点余数；改 GPU 核（WGSL）还是改 CPU 参照先想清楚是哪一种，直接改断言等于把对拍关掉；"
    echo "                没卡的机器上对拍该打一行 NOT_RUN_NO_GPU_ADAPTER 之后返回成功，panic 就是那条用例没接这个约定。"
    return 1
  fi
  # 跑到的条数：日志里数到的加上键相同而复用的（复用的没跑、不在日志里）
  count_tests_run_by_file "$log" > "$scratch/product-gpu.counts"
  awk -F'\t' 'NF >= 7 && $2 == "reuse" && $5 ~ /^tests\// { count[$5]++ } END { for (source in count) printf "%s\t%d\n", substr(source, 7), count[source] }' "$plan_output.kept" > "$scratch/product-gpu.counts.reused"
  for row in "${target_rows[@]}"; do
    file_name="$(basename "${row#*$'\t'}")"
    count="$(awk -F'\t' -v name="$file_name" '$1 == name { print $2 }' "$scratch/product-gpu.counts")"
    count=$(( ${count:-0} + $(awk -F'\t' -v name="$file_name" '$1 == name { total += $2 } END { print total + 0 }' "$scratch/product-gpu.counts.reused") ))
    ran_total=$((ran_total + count))
    if [[ "$count" == 0 && "$crash_point_replay_break" != product-gpu-zero-ignored ]]; then
      zero_files+=("$package_directory/${row#*$'\t'}")
      crash_point_replay_report_not_run "产品 GPU 包 $package_directory/${row#*$'\t'} 挂特性 $feature，真显卡对拍跑到它的用例 0 条（cargo 输出里没有它那一段，或只有 ignored）：给它补一条不标 ignored 的对拍用例，或查它的 required-features 与 cfg"
    fi
  done
  if [[ "$crash_point_replay_break" != product-gpu-not-run-ignored ]]; then
    while IFS=$'\t' read -r gpu_where gpu_line; do
      [[ -n "$gpu_line" ]] || continue
      gpu_not_run+=("$gpu_line")
      crash_point_replay_report_not_run "产品 GPU 包真显卡对拍没跑（${gpu_where}；这台机器找不到 GPU 适配器）：${gpu_line}；在有卡的机器上跑这一格补上"
    done < <(gpu_adapter_not_run_lines "$log")
  fi
  # 逐条认结局与记号（入参在判红那一段之前定好），判绿的照键记库
  if (( ran )) && [[ "$crash_point_replay_break" != product-gpu-cell-skipped ]] && account_test_results "$log" "$serial" "$accounting"; then accounting_ok=1; fi
  if (( plan_ok && ran && accounting_ok )); then
    record_green_tests "$kind" "$plan_file" "$plan_output" "$accounting" "$cell" "cargo test --release -p $package --features $feature（真显卡对拍）" || plan_red=1
  fi
  crash_point_replay_unavailable_only "$cell" && return 1   # 记库那一步 prune-server 不可用：不可用那一句已经打了，不是失灵
  if (( plan_red )); then
    echo "  ✗ $cell 这一格判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；真显卡对拍那一趟照旧全部现判跑完了"
    echo "     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再只跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $cell；不能复用就判红，不退回全部现判当绿"
    return 1
  fi
  not_run_total=$(( ${#zero_files[@]} + ${#gpu_not_run[@]} ))
  if (( not_run_total > 0 )); then
    product_gpu_cell_state="有 ${not_run_total} 处本次未跑"
    echo "  ! $cell 不算全绿：${#target_rows[@]} 个对拍测试目标（特性 $feature）跑到 ${ran_total} 条，本次未跑 ${not_run_total} 处：编进来 0 条的目标 ${#zero_files[@]} 个（${zero_files[*]:-无}），没卡没跑的对拍 ${#gpu_not_run[@]} 条"
    echo "     → 编进来 0 条的给它补不标 ignored 的对拍用例；没卡没跑的在有卡的机器上补跑：bash .claude/gate.d/crash-point-replay.sh --check $cell"
  else
    product_gpu_cell_state="绿"
    echo "  ✓ $cell：${#target_rows[@]} 个对拍测试目标（特性 $feature）每个都跑到了用例，共 ${ran_total} 条（含复用；$( (( plan_ok )) && printf '按条：%s' "$(plan_report "$plan_output.kept" 0)" || printf '按条复用算不出，全部现判')）"
  fi
  return 0
}

# ── 出口 4 那五格（文件头「出口 4 那五格」）─────────
# 判法在 scripts/crash/crash-flow-coverage.py（覆盖、ignore、0 次）与 scripts/crash/crash-case-check.py（must-red、knobs）；这里只备行、调判法、记结局。
crash_point_replay_flow_judge="$crash_point_replay_stage_repository/scripts/crash/crash-flow-coverage.py"
crash_point_replay_flow_records=""            # 这一趟的记录文件（在这一趟的日志目录里）
crash_point_replay_flow_records_problem=""    # 记录备不齐（crash-case-replay 格没跑、判红）：非空时覆盖、ignore、0 次三格本次未跑
crash_point_replay_flow_records_failed=""     # 判法脚本备记录时出错：非空时那三格判红
crash_point_replay_flow_records_unavailable="" # 判法脚本备记录时 prune-server 不可用（collect 退 75）：非空时那三格照不可用办，不判红
crash_point_replay_flow_subset=()             # 点了 --item 时是 (--subset)
crash_point_replay_plain_plan_ok=0
crash_point_replay_feature_plan_ok=0
crash_audit_states=()                           # 一格一项「<格名>：绿 / 红 / 本次未跑（为什么）」
crash_audit_red=()
crash_point_replay_unavailable_cells=()         # test-plan 或 verdict-record 退 75 的格（plan_failure_verdict 记）
crash_point_replay_red_regardless_cells=()      # 其中本来就判红的格（记库那一步不可用之前已经红了）：照红算

# collect_flow_pass <种类> <计划输出|-> <日志> <哪一趟>：一趟按条复用的日志按测试二进制转成记录追加进 crash_point_replay_flow_records
# （crash-flow-coverage.py collect：整份复用的取回判定库里存着的行，一半复用而库里没有的记 unknown）。返回 1 的两种：
#   它退 75（存取覆盖行时 prune-server 不可用）记进 crash_point_replay_flow_records_unavailable，覆盖、ignore、0 次三格照不可用办（文件头「退出码」75）；
#   别的非 0 是判法脚本出错，记进 crash_point_replay_flow_records_failed，那三格判红。弄坏开关 flow-records-unavailable-as-failure 下 75 也照出错办（改前的样子）。
#   样本档标记目录里放了 collect-unreachable 的，这一步连一处连不上的服务（socket 不在、TCP 端口刚放掉、重连总时长 2 秒）造 75（文件头「样本档」）
collect_flow_pass() {
  local collect_exit unreachable_settings=()
  if [[ "$crash_point_replay_sample_mode" == 1 && -f "$PWD/.crash-point-replay-sample-tools/collect-unreachable" ]]; then
    # 改连接地址的变量只在临时服务底下认（scripts/prune-server/prune_server_client.py 文件头「连哪一处」）：临时服务的标记一并设上、指着同一个不在的 socket
    unreachable_settings=(env "SINGLEFS_PRUNE_SERVER_SOCKET=$crash_point_replay_scratch_directory/no-prune-server.sock"
      "PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET=$crash_point_replay_scratch_directory/no-prune-server.sock" SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS=2
      "SINGLEFS_PRUNE_SERVER_TCP_ADDRESS=127.0.0.1:$(python3 -c 'import socket; listener = socket.socket(); listener.bind(("127.0.0.1", 0)); print(listener.getsockname()[1]); listener.close()')")
  fi
  if ${unreachable_settings[@]+"${unreachable_settings[@]}"} python3 "$crash_point_replay_flow_judge" collect "$ROOT" "$1" "$2" "$3" --pass "$4" \
       >>"$crash_point_replay_flow_records" 2>"$crash_point_replay_flow_records.err"; then collect_exit=0; else collect_exit=$?; fi
  cat "$crash_point_replay_flow_records.err"
  if (( collect_exit == 75 )) && [[ "$crash_point_replay_break" != flow-records-unavailable-as-failure ]]; then
    crash_point_replay_flow_records_unavailable="crash-flow-coverage.py collect 退 75（$4）：$(tail -n 2 "$crash_point_replay_flow_records.err" | tr '\n' ' ' | cut -c1-300)"
    return 1
  fi
  if (( collect_exit != 0 )); then
    crash_point_replay_flow_records_failed="crash-flow-coverage.py collect 退 ${collect_exit}（$4）：$(tail -n 2 "$crash_point_replay_flow_records.err" | tr '\n' ' ' | cut -c1-300)"
    return 1
  fi
  return 0
}

# prepare_flow_records：按条复用两趟（crash-case-replay 格与 feature-gated-tests 格）的日志与这一趟判绿的每条崩溃枚举用例的日志转成记录（文件头「出口 4 那五格」），
# 记录文件放这一趟的日志目录（flow-records.tsv，跑完不删），打一行 CRASH_FLOW_RECORDS_FILE <路径>（崩溃点重放入口照这一行把它并进覆盖表）；
# 没判绿的用例记 unknown。点了 --item 时带 --subset（入口没有行、成员没被罩住两样本次未跑）。
prepare_flow_records() {
  local plain_plan="-" feature_plan="-" feature_log="$crash_point_replay_feature_pass_log" row key package target function log
  crash_point_replay_flow_records="$crash_point_replay_log_directory/flow-records.tsv"
  : >"$crash_point_replay_flow_records"
  (( ${#crash_point_replay_named_cases[@]} > 0 )) && crash_point_replay_flow_subset=(--subset)
  [[ -f "$crash_point_replay_flow_judge" ]] || return 0
  if ! crash_point_replay_runs_cell "$CRASH_CASE_REPLAY_CELL_NAME"; then
    crash_point_replay_flow_records_problem="这一趟没跑 $CRASH_CASE_REPLAY_CELL_NAME（--check 没点它），按条复用那一趟与用例的覆盖行与计数行没有；连它一起点：--check $CRASH_CASE_REPLAY_CELL_NAME,$FEATURE_GATED_TESTS_CELL_NAME,<这一格>"
    return 0
  fi
  if (( ! crash_point_replay_cell_green )); then
    crash_point_replay_flow_records_problem="$CRASH_CASE_REPLAY_CELL_NAME 判红，覆盖行与计数行不全"
    return 0
  fi
  (( crash_point_replay_plain_plan_ok )) && plain_plan="$crash_point_replay_scratch_directory/plain.plan.out"
  collect_flow_pass "$(crash_point_replay_verdict_kind "$crash_point_replay_stage_file_name" "$CRASH_CASE_REPLAY_CELL_NAME")" "$plain_plan" \
    "$crash_point_replay_plain_pass_log" "不带特性那一趟" || return 0
  if ! crash_point_replay_runs_cell "$FEATURE_GATED_TESTS_CELL_NAME"; then
    # 没点带特性那一格：有挂特性的测试文件，它们打的行这一趟没有，记行不全；一份都没有就没缺什么
    if [[ -n "$(list_feature_gated_test_files crates/singlefs-verification/tests 2>/dev/null)" ]]; then
      printf 'CRASH_FLOW_RECORD\tunknown\tfeature-gated-tests\t这一趟没跑带特性那一趟（--check 没点 %s），挂特性的测试文件打的行没有\n' "$FEATURE_GATED_TESTS_CELL_NAME" >>"$crash_point_replay_flow_records"
    fi
  elif [[ "$feature_cell_outcome" == red ]]; then
    printf 'CRASH_FLOW_RECORD\tunknown\tfeature-gated-tests\t带特性那一趟判红，挂特性的测试文件打的行不全\n' >>"$crash_point_replay_flow_records"
  elif [[ -f "$feature_log" ]]; then
    (( crash_point_replay_feature_plan_ok )) && feature_plan="$crash_point_replay_scratch_directory/features.plan.out.kept"
    collect_flow_pass "$(crash_point_replay_verdict_kind "$crash_point_replay_stage_file_name" "$FEATURE_GATED_TESTS_CELL_NAME")" "$feature_plan" "$feature_log" "带特性那一趟" || return 0
  fi
  for row in "${crash_case_rows[@]}"; do
    IFS=$'\t' read -r key package target function <<< "$row"
    log="$crash_point_replay_log_directory/log.${key#crash-case:}"
    if crash_point_replay_cell_in "$key" ${green_cases[@]+"${green_cases[@]}"} && [[ -f "$log" ]]; then
      if ! python3 "$crash_point_replay_flow_judge" extract --source "$target" "$log" >>"$crash_point_replay_flow_records" 2>"$crash_point_replay_flow_records.err"; then
        crash_point_replay_flow_records_failed="crash-flow-coverage.py extract 读 $log 出错：$(tail -n 2 "$crash_point_replay_flow_records.err" | tr '\n' ' ' | cut -c1-300)"
        return 0
      fi
    else
      printf 'CRASH_FLOW_RECORD\tunknown\t%s\t%s 这一趟没判绿（%s），它打的行不算\n' "$target" "$key" "$function" >>"$crash_point_replay_flow_records"
    fi
  done
  [[ -s "$crash_point_replay_flow_records" ]] || return 0
  echo "  · 覆盖行与计数行：$(grep -c . "$crash_point_replay_flow_records") 条记录，记在 $crash_point_replay_flow_records（填两张基准的初值：python3 scripts/crash/crash-flow-coverage.py baseline <ignored|zero-evaluations> 这一份）"
  echo "CRASH_FLOW_RECORDS_FILE $crash_point_replay_flow_records"
  return 0
}

# run_crash_audit_cell <格名>：跑出口 4 的一格，判法的输出原样转打；结局并进 crash_audit_states，红的进 crash_audit_red，
# 本次未跑的往 $GATE_NOT_RUN_FILE 记一行。判法退 0 绿、1 红、77 本次未跑，别的退出码当判法出错、判红
run_crash_audit_cell() {
  local cell="$1" cell_exit reason output="$crash_point_replay_scratch_directory/crash-audit.$1.out"
  crash_audit_command=()
  crash_audit_not_run_reason=""
  if [[ "$cell" == crash-case-must-red || "$cell" == crash-knobs || "$cell" == "$CRASH_CASE_PROVENANCE_CELL_NAME" ]]; then echo "── $cell（只读树）"; else echo "── $cell"; fi
  case "$cell" in
    crash-flow-coverage|crash-ignored-nodes|invariant-zero-evaluations)
      if [[ ! -f "$crash_point_replay_flow_judge" ]]; then
        crash_audit_not_run_reason="判法脚本 $crash_point_replay_flow_judge 不在（这一份阶段被拷到了不带 scripts/ 的树里）"
      elif [[ -n "$crash_point_replay_flow_records_unavailable" ]]; then
        # 覆盖行存取连不上 prune-server：这一格照不可用办（进 crash_point_replay_unavailable_cells），整道收尾时只有不可用退 75、另有真红退 1（文件头「退出码」）
        echo "  ✗ $cell 这一格没判：prune-server 不可用（$crash_point_replay_flow_records_unavailable）；别的格照跑（不是红，也不算通过）"
        echo "     → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；修好再连 crash-case-replay 一起重跑：--check $CRASH_CASE_REPLAY_CELL_NAME,$FEATURE_GATED_TESTS_CELL_NAME,$cell"
        crash_point_replay_cell_in "$cell" ${crash_point_replay_unavailable_cells[@]+"${crash_point_replay_unavailable_cells[@]}"} \
          || crash_point_replay_unavailable_cells+=("$cell")
        crash_audit_states+=("$cell：prune-server 不可用"); crash_audit_red+=("$cell")
        return 0
      elif [[ -n "$crash_point_replay_flow_records_failed" ]]; then
        echo "  ✗ $cell：$crash_point_replay_flow_records_failed"
        echo "     → 怎么办：照那一句修 scripts/crash/crash-flow-coverage.py（单跑它的 --selftest 看是哪一格），再连 crash-case-replay 一起重跑：--check $CRASH_CASE_REPLAY_CELL_NAME,$cell"
        crash_audit_states+=("$cell：红"); crash_audit_red+=("$cell")
        return 0
      elif [[ -n "$crash_point_replay_flow_records_problem" ]]; then
        crash_audit_not_run_reason="$crash_point_replay_flow_records_problem"
      else
        local subcommand="coverage"
        [[ "$cell" == crash-ignored-nodes ]] && subcommand="ignored"
        [[ "$cell" == invariant-zero-evaluations ]] && subcommand="zero-evaluations"
        crash_audit_command=(python3 "$crash_point_replay_flow_judge" check "$subcommand" "$ROOT" ${crash_point_replay_flow_subset[@]+"${crash_point_replay_flow_subset[@]}"} "$crash_point_replay_flow_records")
      fi ;;
    crash-case-must-red|crash-knobs|crash-case-provenance)
      if [[ ! -f "$crash_point_replay_static_judge" ]]; then
        crash_audit_not_run_reason="判法脚本 $crash_point_replay_static_judge 不在（这一份阶段被拷到了不带 scripts/ 的树里）"
      else
        local item="must-red"
        [[ "$cell" == crash-knobs ]] && item="knobs"
        [[ "$cell" == crash-case-provenance ]] && item="provenance"
        crash_audit_command=(python3 "$crash_point_replay_static_judge" --only "$item" "$ROOT")
        # 弄坏开关 provenance-judge-not-run：来历表核对这一格不起判法，直接记绿
        [[ "$cell" == crash-case-provenance && "$crash_point_replay_break" == provenance-judge-not-run ]] && crash_audit_command=(true)
      fi ;;
  esac
  if (( ${#crash_audit_command[@]} > 0 )); then
    if "${crash_audit_command[@]}" >"$output" 2>&1; then cell_exit=0; else cell_exit=$?; fi
    cat "$output"
    case "$cell_exit" in
      0) crash_audit_states+=("$cell：绿"); return 0 ;;
      77) crash_audit_not_run_reason="$(grep -m 1 '本次未跑' "$output" | sed 's/^[[:space:]]*! 本次未跑（这不是通过）：//' | python3 -c 'import sys; print(sys.stdin.read().strip()[:120])')"
          [[ -n "$crash_audit_not_run_reason" ]] || crash_audit_not_run_reason="判法退 77（无对象可判）" ;;
      1) crash_audit_states+=("$cell：红"); crash_audit_red+=("$cell"); return 0 ;;
      *)
        echo "  ✗ $cell：判法 ${crash_audit_command[*]:0:3} 退 ${cell_exit}（不是 0、1、77：判法自己出错，输出在上面）"
        echo "     → 怎么办：照上面的报错修那份判法脚本，单跑同一条命令到它退 0、1 或 77，再只重跑这一格"
        crash_audit_states+=("$cell：红"); crash_audit_red+=("$cell"); return 0 ;;
    esac
  fi
  reason="${crash_audit_not_run_reason:-没说为什么}"
  [[ -s "$output" ]] || echo "  ! 本次未跑（这不是通过）：$cell：$reason"
  crash_audit_states+=("$cell：本次未跑（${reason}）")
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "crash-point-replay.sh 的 $cell 格本次未跑：$reason" >> "$GATE_NOT_RUN_FILE"; fi
  return 0
}

# make_log_directory：每趟一个新目录（不拿上一趟的输出顶上），放每条崩溃枚举用例的日志与发现日志、覆盖行记录和全绿标记；头一次用到才建，跑完不删
crash_point_replay_log_directory=""
make_log_directory() {
  [[ -z "$crash_point_replay_log_directory" ]] || return 0
  crash_point_replay_log_directory="$git_common_directory/singlefs-crash-point-replay-logs/$(date -u +%Y%m%dT%H%M%SZ)-$$"
  if ! mkdir -p -- "$crash_point_replay_log_directory"; then
    echo "  ✗ 建不了这一趟的日志目录 $crash_point_replay_log_directory：每条用例的日志与发现日志没处放"
    echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后重跑这一道。"
    exit 1
  fi
  echo "  · 这一趟每条用例的日志与发现日志放 ${crash_point_replay_log_directory}/（log.<用例名> 与 log.<用例名>.findings.tsv，跑完不删）"
}

# 登记的崩溃枚举用例与放量分项各落到哪个「测试目标、用例函数」（按条复用的几趟用 --skip 排掉它们）：用没按 --item 筛过的登记取，取不出整道红
crash_point_replay_registered_pairs_file="$crash_point_replay_scratch_directory/registered-functions.tsv"
if registered_test_functions >"$crash_point_replay_registered_pairs_file.raw" 2>"$crash_point_replay_registered_pairs_file.err"; then
  LC_ALL=C sort -u "$crash_point_replay_registered_pairs_file.raw" >"$crash_point_replay_registered_pairs_file"
else
  echo "  ✗ 登记的崩溃枚举用例与放量分项各落到哪个用例函数，取不出来（$(tail -n 2 "$crash_point_replay_registered_pairs_file.err" | tr '\n' ' ' | cut -c1-300)）：按条复用那几趟排不掉它们"
  echo "     → 怎么办：单跑 python3 scripts/crash/lib_crash_point_replay_items.py check <根> 看合表与放量入口的 ITEMS 哪里读不了，修好之后重跑这一道。"
  exit 1
fi

# ── 格：crash-case-replay、feature-gated-tests、product-gpu-real-card-tests，再是出口 4 那五格 ─────────
red_cells=()
cell_states=()
package_pass_log=""
crash_point_replay_cell_green=0
crash_point_replay_cell_state="绿"
feature_cell_outcome=""
red_cases=(); green_cases=(); reused_cases=()
if crash_point_replay_runs_cell "$CRASH_CASE_REPLAY_CELL_NAME"; then
  package_pass_report=""
  package_pass_plan_red=0
  make_log_directory
  # 按块判定红了是真红：记库那一步 prune-server 不可用之后块又红的，照红算（crash_point_replay_red_regardless_cells）
  if run_package_test_functions && { run_registered_crash_cases || { crash_point_replay_red_regardless_cells+=("$CRASH_CASE_REPLAY_CELL_NAME"); false; }; } && package_pass_reuse_held; then
    (( package_pass_ran )) && package_pass_log="$crash_point_replay_plain_pass_log"
    crash_point_replay_cell_green=1
  else
    # 这一格红了：不带特性那一趟的本次未跑不补判、不报（红句已经点了出路）
    : >"$crash_point_replay_plain_not_run_file"
    red_cells+=("$CRASH_CASE_REPLAY_CELL_NAME")
  fi
  flush_deferred_pass_outcomes
fi
if crash_point_replay_runs_cell "$FEATURE_GATED_TESTS_CELL_NAME"; then
  if run_feature_gated_tests_cell "$package_pass_log"; then
    feature_cell_outcome="ran"
  else
    feature_cell_outcome="red"
    red_cells+=("$FEATURE_GATED_TESTS_CELL_NAME")
  fi
  flush_deferred_pass_outcomes
fi
# 产品 GPU 包的真显卡对拍（文件头「product-gpu-real-card-tests 格」）：与上面两趟互不依赖，不进覆盖、ignore、0 次那几格的行
product_gpu_cell_outcome=""
product_gpu_cell_state=""
if crash_point_replay_runs_cell "$PRODUCT_GPU_REAL_CARD_CELL_NAME"; then
  if run_product_gpu_real_card_cell; then
    product_gpu_cell_outcome="ran"
  else
    product_gpu_cell_outcome="red"
    red_cells+=("$PRODUCT_GPU_REAL_CARD_CELL_NAME")
  fi
fi
if crash_point_replay_runs_cell "$CRASH_CASE_REPLAY_CELL_NAME"; then
  if (( crash_point_replay_cell_green )); then
    # 不带特性那一趟用例报的本次未跑（文件头「多机配置开了 GPU」）：带特性那一趟补判之后仍没判到的逐条报，逐格汇总里点出来，不算全绿
    report_plain_pass_not_run "$feature_cell_outcome"
    if (( crash_point_replay_plain_not_run_after > 0 )); then
      crash_point_replay_cell_state="绿；不带特性那一趟有 ${crash_point_replay_plain_not_run_after} 处本次未跑"
    fi
    cell_states+=("$CRASH_CASE_REPLAY_CELL_NAME：$crash_point_replay_cell_state")
  else
    cell_states+=("$CRASH_CASE_REPLAY_CELL_NAME：$(red_cell_state "$CRASH_CASE_REPLAY_CELL_NAME")")
  fi
fi
if [[ "$feature_cell_outcome" == ran ]]; then
  cell_states+=("$FEATURE_GATED_TESTS_CELL_NAME：$feature_cell_state")
elif [[ "$feature_cell_outcome" == red ]]; then
  cell_states+=("$FEATURE_GATED_TESTS_CELL_NAME：$(red_cell_state "$FEATURE_GATED_TESTS_CELL_NAME")")
fi
if [[ "$product_gpu_cell_outcome" == ran ]]; then
  cell_states+=("$PRODUCT_GPU_REAL_CARD_CELL_NAME：$product_gpu_cell_state")
elif [[ "$product_gpu_cell_outcome" == red ]]; then
  cell_states+=("$PRODUCT_GPU_REAL_CARD_CELL_NAME：$(red_cell_state "$PRODUCT_GPU_REAL_CARD_CELL_NAME")")
fi
# 出口 4 那五格（文件头「出口 4 那五格」）：覆盖、ignore、0 次三格读上面两趟与每条用例的行；点了 --check 的只跑点名的
flow_cells_wanted=0
for crash_audit_cell in "${CRASH_AUDIT_FLOW_CELLS[@]}"; do crash_point_replay_runs_cell "$crash_audit_cell" && flow_cells_wanted=1; done
if (( flow_cells_wanted )); then make_log_directory; prepare_flow_records; fi
for crash_audit_cell in "${CRASH_AUDIT_CELL_NAMES[@]}" "$CRASH_CASE_PROVENANCE_CELL_NAME"; do
  if crash_point_replay_runs_cell "$crash_audit_cell"; then run_crash_audit_cell "$crash_audit_cell"; fi
done
cell_states+=(${crash_audit_states[@]+"${crash_audit_states[@]}"})
[[ "$crash_point_replay_break" == crash-audit-red-ignored ]] && crash_audit_red=()   # 弄坏开关：出口 4 那几格红了也不算进整道
red_cells+=(${crash_audit_red[@]+"${crash_audit_red[@]}"})
cell_summary=""
for cell_state in "${cell_states[@]}"; do cell_summary+="${cell_summary:+；}${cell_state}"; done
echo "  · 逐格：${cell_summary}"
# 红的格分两堆（文件头「退出码」）：只因 prune-server 不可用判不了的（crash_point_replay_unavailable_only）与真红的；有真红退 1、两堆都报，只有不可用退 75
unavailable_cells=()
truly_red_cells=()
for red_cell in ${red_cells[@]+"${red_cells[@]}"}; do
  if crash_point_replay_unavailable_only "$red_cell"; then unavailable_cells+=("$red_cell"); else truly_red_cells+=("$red_cell"); fi
done
if (( ${#truly_red_cells[@]} == 0 && ${#unavailable_cells[@]} > 0 )); then
  echo "  ✗ prune-server 不可用：${#unavailable_cells[@]} 格连不上判定库、没判：${unavailable_cells[*]}（不是红，也不算通过）"
  echo "     → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；修好再只重跑这几格：bash .claude/gate.d/crash-point-replay.sh --check $(rerun_check_list "${unavailable_cells[@]}") [项目根]"
  exit 75
fi
if (( ${#red_cells[@]} > 0 )); then
  echo "  ✗ 有 ${#truly_red_cells[@]} 格判红：${truly_red_cells[*]}（各自的出路在上面）"
  (( ${#unavailable_cells[@]} > 0 )) && echo "     另有 ${#unavailable_cells[@]} 格 prune-server 不可用、没判：${unavailable_cells[*]}（python3 prune-server/control check 看服务，起不来报给主 agent）"
  echo "     → 怎么办：照上面那几格各自的「→ 怎么办」改完，只重跑红的格：bash .claude/gate.d/crash-point-replay.sh --check $(rerun_check_list "${red_cells[@]}") [项目根]"
  exit 1
fi
crash_point_replay_pass_logs_disposable=1   # 判绿收尾：每一趟 cargo 的整份输出不用留（文件头「每一趟当场写结局」；每条用例的整份输出另在跑完不删的日志目录里，不受这一行管）
# 缩过域的这一趟（环境里设了展开深度变量，引擎只往小缩）：判绿也不算门禁通过，不写 all-green；点没点范围都写出设了的变量与值
crash_point_replay_narrowed=""
for crash_point_replay_narrowing_variable in SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO; do
  [[ -n "${!crash_point_replay_narrowing_variable+x}" ]] || continue
  crash_point_replay_narrowed+="${crash_point_replay_narrowed:+、}$crash_point_replay_narrowing_variable=${!crash_point_replay_narrowing_variable}"
done
[[ "$crash_point_replay_break" == narrowed-run-writes-all-green ]] && crash_point_replay_narrowed=""   # 弄坏开关：不看缩域变量（改前的样子）
if [[ -n "$crash_point_replay_narrowed" ]]; then
  echo "  · 缩过域、不算门禁通过：$crash_point_replay_narrowed（只往小缩，大于引擎常量的不放宽）；不写 all-green，scripts/gate/apply-writer-patch.py 数上次崩溃验证全绿以来打了几个补丁时这一趟不算全绿"
fi
# 不指定范围（没给 --check，也没给 --item）而且没缩过域判绿的这一趟：日志目录里留一份 all-green（只记这一趟绿过，scripts/gate/apply-writer-patch.py 数「上次崩溃验证全绿以来打了几个补丁」用；不是复用的标记）
if (( ! crash_point_replay_range_given )) && [[ -z "$crash_point_replay_narrowed" ]] && [[ "$crash_point_replay_break" != all-green-not-written ]] && (( crash_point_replay_cell_green )) && [[ -n "$crash_point_replay_log_directory" ]]; then
  if printf '%s\n' "$(TZ=Asia/Tokyo date +%F) ${#crash_case_rows[@]} 条崩溃枚举用例全部判绿" > "$crash_point_replay_log_directory/all-green"; then
    echo "  · 不指定范围，这一趟判绿：$crash_point_replay_log_directory/all-green 记下了这一趟（apply-writer-patch.py 数上次崩溃验证全绿以来打了几个补丁用）"
  else
    echo "  ! 没写成 $crash_point_replay_log_directory/all-green（apply-writer-patch.py 的提示照上一趟全绿算）"
  fi
fi
if [[ "$crash_point_replay_sample_mode" == 1 ]]; then
  echo "  ! 样本档（.crash-point-replay-sample-tools，假 cargo）：流程判绿，退 3 不退 0——这一趟没跑过真的崩溃点重放"
  echo "     → 真跑要在不带 .crash-point-replay-sample-tools 的树里跑这一道。"
  exit 3
fi
exit 0
