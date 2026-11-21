#!/usr/bin/env bash
# task-server-shards: *
# task-server-compiles: yes
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 scripts/gate/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判
# gate-stage: 崩溃点重放与登记的崩溃枚举用例（包里全部测试函数按单元键复用，登记的崩溃枚举用例逐条按崩溃状态块的单元键跑，登记行第三列带 separate-run= 的除外）
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: code-source-discipline.sh 与 model-differential-and-scenarios.sh 也调 scripts/crash/crash-case-check.py，但调的是 file-names、one-scenario 两样，分归代码类与 harness 类；这里开跑前只调 placement、modules 两样，判的是崩溃枚举用例与 验证包测试文件，红了就不起 cargo；crash-case-must-red、crash-knobs、crash-case-provenance 三格另调 must-red、knobs、provenance 三样（崩溃枚举用例的必红变异、崩溃点测试的旋钮表、合表各项在来历表里的行，别的门禁都不调）
# gate-covers: 崩溃点重放
# gate-cell: crash-case-replay 包里全部测试函数按单元键复用（登记的崩溃枚举用例经 --skip 排掉）、登记的崩溃枚举用例逐条按崩溃状态块的单元键跑（块键相同的复用、缺的当场判；登记行第三列带 separate-run= 的除外，没被 --item 点名时逐条报本次未跑）；带 --crash-cases-only 时只走 --item 点名的崩溃枚举用例按块那一路、不起按条复用那一趟
# gate-cell: product-gpu-real-card-tests 起产品 GPU 包的测试包（singlefs-background-gpu-tests）带 tests-on-a-real-gpu 特性才编的真显卡对拍测试（每条测试函数按单元键复用）：对拍判红整格红，没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑
# gate-cell: crash-case-must-red 每条崩溃枚举用例在 crates/mutations.tsv 里至少有一行变异点名它的测试目标
# gate-cell: crash-knobs 旋钮登记表 crash-knobs.tsv 每行取值与依据不空、出处路径在
# gate-cell: crash-case-provenance 崩溃点重放合表里调 enumerate_crash_states 的崩溃枚举项与放量分项，在来历表 crash_point_replay_provenance.rs 里都有一行
# gate-cell: feature-gated-tests 带上 验证包 tests/ 里 cfg 点到的特性另跑一趟（每条测试函数按单元键复用）：挂特性的测试文件逐个跑到或复用的用例不为 0，GPU 对拍没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑
# gate-cell: crash-flow-coverage 崩溃流覆盖表：没有登记表 crash-entries.tsv 外的入口、登记的入口都有覆盖行、StepKind 每个成员都被一条流罩住
# gate-cell: crash-ignored-nodes ignore 开着的节点（ignored_point、registered_segment、not_expanded）不比基准 crash-ignored-nodes.tsv 多
# gate-cell: invariant-zero-evaluations 评估 0 次的（流，不变量）对都在带理由的基准 invariant-zero-evaluations.tsv 里
#
# 只有一条路：不带 --check 与 --item 就是九格全跑，门禁照单元键自己选要判的（键相同的复用，键对不上的现判），没有轻重、快慢之分，也没有「全部现判」的跑法。
# 格的次序（格名表的次序就是跑的次序）：要跑得久的排在后面，别的格先出结局、先记判定，整道被停了也留得下：
#   crash-case-replay（包里按条复用那一趟、再是登记的崩溃枚举用例）、product-gpu-real-card-tests、只读树的 crash-case-must-red、crash-knobs、crash-case-provenance、feature-gated-tests、
#   最后是读前面几趟的行的 crash-flow-coverage、crash-ignored-nodes、invariant-zero-evaluations。
#   feature-gated-tests 这一格里，FEATURE_PASS_LAST_TARGETS 的目标（crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state，要跑的时间比别的加起来长得多）另起的一趟排在最后；
#   它不单独成格：要改 feature-gated:<文件> 的项归哪一格、--check 的写法与 .claude/rules/verification.md 里这一趟的写法，判定、键与种类也都不因为它在哪一趟而变。
#   带特性那一趟一收尾就先把它的判定记下（record_feature_main_pass_first），不等排在最后的那一趟；这一格收尾时把两趟的判定再记一遍，同键同结论，复用哨兵在这一遍比。
#   一格一收尾打一行「· 这一格先出结局：<格名>：<结局>」（crash_point_replay_announce_cell）；带特性那一格的结局在它自己的 ✓、✗ 与末尾逐格那一行。
#   判定、单元键与种类不随次序变；逐格汇总与红格清单的次序不跟着变（先是起 cargo 的三格，再是覆盖、ignore、0 次、必红世界、旋钮、来历表）。
#   弄坏开关 last-pass-before-the-other-cells 下带特性那一格排在产品 GPU 与只读树的格之前；feature-main-pass-recorded-with-last-pass 下不先记。样本 last-pass-runs-after-the-other-cells 钉这两条。
# 红了接着跑，停了不接着跑（crash-case-replay 格两部分的红都要看到，一处红不把一百多条用例挡在外面）：
#   不带特性那一趟判红，登记的崩溃枚举用例照跑，整格照样判红，两部分的红都报；两部分都红时另打一行「✗ crash-case-replay 两部分都红」。弄坏开关 package-pass-red-skips-cases 下红了不跑用例。
#   不接着跑的两种：prune-server 不可用（用例的块判定同样读写它）；这一趟被停了（crash_point_replay_note_stop）：起 cargo 的那一层退 130、137、143，
#   或日志里 cargo 报测试二进制死于 SIGINT、SIGKILL、SIGTERM（逐个停进程、撞内存上限都是这样，cargo 这时退 101）；死于别的信号（SIGSEGV、SIGABRT）是测试自己崩了，照红、接着跑。
#   被停之后，要起 cargo 的格、后面的用例、排在最后的那一趟与读行的三格都不接着跑，逐个报本次未跑；后面的用例 ITEM_DONE 报 exit=77；门禁自己收到信号时 bash 当场退，走不到这里。
#   弄坏开关 stopped-pass-runs-the-rest 下不认被停。样本 plain-pass-red-crash-cases-still-run、plain-pass-red-and-crash-case-red-both-reported、plain-pass-stopped-skips-the-rest、
#   plain-pass-test-binary-terminated-skips-the-rest、crash-case-stopped-skips-the-rest、features-main-pass-stopped-skips-last-pass 钉这一条
#   （没有分片阶段时除 features-main-pass-stopped-skips-last-pass 外，这几份样本里的红与被停都发生在单片那一步，整道退单片的码、后面的格与合并不跑）。
# 指定范围只有两种：--check 点名格，--item 点名项。项有两类（下面「逐项点名」）：崩溃枚举用例（只限定按块那一路跑哪几条，按条复用那一趟照跑；带 --crash-cases-only 时不跑，见下）、
#   挂特性的测试文件（feature-gated:<测试文件名>，只限定 feature-gated-tests 格编哪几个、跑哪几个）。
# --crash-cases-only：只与点名崩溃枚举用例的 --item 同给，日常只判点名的用例、不起整个验证包（scripts/gate/merge-and-verify.sh 的验证包那一步这样起）。
#   带上它，crash-case-replay 格只走按块那一路：只跑 --item 点名的崩溃枚举用例（判法、块键复用、CRASH_CASE_BLOCKS 那一行照下面「复用」一段），
#   包里全部测试函数按条复用那一趟不起：不算计划、不起 cargo、不记库；逐格汇总里这一格写「绿（--crash-cases-only：只判了按块那一路，按条复用那一趟没起）」。
#   别的格不受它影响（同写在 --check 里的照常跑）；出口 4 读覆盖行的三格把按条复用那一趟记一条 unknown（行不全）。
#   没点崩溃枚举用例（没给 --item，或只点了 feature-gated 的项）退 2。样本根的 .gate-crash-cases-only（内容不读）当作命令行上的 --crash-cases-only，
#   认法同下面 .gate-items（只在样本根上、命令行没给时认）。
#
# 逐项点名（.claude/rules/test-script-discipline.md「入口脚本：不带参数就是不指定范围」，统一任务服务按项估时、分片读下面两种机读行）：
#   项的写法：
#     crash-case:<用例名>               登记的崩溃枚举用例（.claude/gate.d/stage-inputs.tsv 键是 crash-case: 的行；--item 里 crash-case: 可写可不写），归 crash-case-replay 格的按块那一路；
#     feature-gated:<测试文件名>        验证包 tests/ 里挂特性 cfg 的一个测试目标（list_feature_gated_test_files 现读；<测试文件名> 是根文件名：平铺的 <名>.rs，目录式的 <名>/main.rs，
#                                       例 feature-gated:gpu_pair.rs、feature-gated:store_only/main.rs），归 feature-gated-tests 格；
#     feature-gated:lib                 那一格带特性那一趟里的 --lib 一段（库在这一组特性下的单测）。
#     feature-gated 的项合起来就是整格：不点就是全部；点了的只编、只跑点名的测试目标（--lib 只在点了 feature-gated:lib 时带），特性照整格那一组
#     （tests/ 里全部挂特性的文件现读的并集），所以每条测试函数的单元键与整格里的相同，点名单跑一项与它在整格里跑的判定相同、判定库里互相复用。
#     不带特性那一趟打了 NOT_RUN_ 的补判只在同一趟跑了 crash-case-replay 格时才有，点了 feature-gated 的项那一趟不另加目标。
#   没给 --check 时：点了 feature-gated 的项而没点崩溃枚举用例的，只跑 feature-gated-tests 一格；点了崩溃枚举用例的照旧九格（feature-gated 的项照样只限定那一格）。
#   给了 --check 而点了 feature-gated 的项、点的格里没有 feature-gated-tests：退 2（点的项不在点的格里）。项写错（没登记的用例、tests/ 里没有挂特性的这个文件）退 2，列出可点的项。
#   样本根的 .gate-items（一行一项，# 之后与空白不算）当作命令行上的 --item：只在被判的根是样本（不是本阶段所在的仓，也没设 GATE_STAGED_FROM）、命令行没给 --item 时认；
#     stage-selftest.sh 起样本只给根，样本要点名就靠它（同门禁 code-tooling 的 .gate-items）。
#   机读行（写法同 scripts/crash/crash-point-replay.sh 文件头「输出」；统一任务服务不读它们）：
#     ITEM_PLAN item=<项> to_judge=<件数> reusable=<件数>   --list-items --with-keys，stdout，列项之后每项一行；同时给了 --item 的只打点名的项。
#       feature-gated 的项：件是那个测试目标里编得进的测试函数（admission.py test-plan 带这一组特性、在清掉 SINGLEFS_CRASH_AMPLIFICATION_* 与
#       SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 的环境里算，登记的崩溃枚举用例与放量分项经 --exclude-registered 不算），键对不上与拒算的算要现判、键相同的算复用；
#       崩溃枚举用例：块键只有引擎按整库数、没有按项的读法，一条用例一件、算要现判（同崩溃点重放入口）。test-plan 算不成的那几项不打 ITEM_PLAN、另打一行「算不出」。
#     ITEM_DONE item=<项> seconds=<秒> judged=<现判的件数> reused=<复用的件数> exit=<0|1|77>   真跑，stdout，跑完每项一行：
#       feature-gated 的项在那一格收尾时打：秒数是这一项那几段 libtest 汇总行「finished in <秒>s」之和（只算测试跑的时间，编译几项共用一份、不分摊；一条都不用跑的项 0），
#       件数照这一格的计划；exit 1 是那一格判红（这一项里有 FAILED 的测试函数，或认不出红在哪一项时点到的每一项都记 1），77 是这一项跑到加复用 0 条或有没卡没跑的 GPU 对拍，别的 0。
#       崩溃枚举用例在按块那一路每条跑完时打：秒数是这一条从开跑到跑完的挂钟，全部块都复用的记 judged=0 reused=1、别的 judged=1 reused=0，判红 exit 1。
#
# 出口 4 那五格（.claude/kb/milestone/03-third-txn.md「出口」的出口 4、六·1、六·3、六·4；K1–K7 七条的原话在
#   records/2026-10-18-里程碑三收尾与近期欠账.md里定出口 4 七条的那一节）：判法不在这里，覆盖、ignore、0 次三格在
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
#   后者经 scripts/crash/lib_crash_point_replay_items.py 的 pipeline_items 读）从 test-plan 的逐条行里去掉，验证包那两趟再经 --skip 排在最前面：登记的崩溃枚举用例交按块那一路
#   （run_registered_crash_cases 只走登记的 crash-case: 行），放量分项这道门禁不跑（它们归崩溃放量入口 scripts/crash/crash-amplification.sh，什么时候起写在 .claude/skills/crash-test/commit-stages.md）。
#   结局经 lib_libtest_result_lines.py 的 recordable_results 逐条认（判法写在它文件头「按条记库」）、判绿的照键记库，按测试函数不记：打了 NOT_RUN_ 的行
#   认作谁打的，那一条不记；判段判在 CPU 的行只在多机配置开了 GPU 时让那一条不记（那一趟判红，同一趟的判定照记：见「多机配置开了 GPU」），没开 GPU 时判在 CPU 的绿照键记库
#   （D13 已定项 17：复用与剪枝不看设备）；结果行被打散的，接得上结局的照记、认不出的那几条不记；完整性闸（没有汇总行、认出的比汇总行报的多）照留，不记的下一趟照跑。
#   记红、撤绿、下趟现判：按条复用的那几趟 cargo test 退非 0 时，report_red_test_targets 抄出的 FAILED 测试函数（结果行、打散后单独一行的结局、libtest 末尾 failures: 名单）
#   按（目标根文件，libtest 全名）对到计划里的那一行，经 admission.py verdict-record --red-names 照计划里的键记红、撤掉同一把键原有的绿（record_red_tests），
#   下一趟这几条现判、原因以「上次判红」起头；scripts/crash/crash-point-replay.sh 按项跑的那一段同一规矩、记进同一份库。
#   有红的那一趟里判绿的照样记绿：同一份输出照上面逐条认（同一套 recordable_results 与完整性闸），认出 ok 的经 plan-green-names 取名单、与记红交同一次
#   verdict-record（record_red_tests 带清单文件），一条别人的红不让同一趟别的绿下一趟重判；认不成、结局认不出、编不过（一条结局行都没有）的一条都不记，下一趟照跑。
#   这一格的结论与出路照旧判红。
#   现判与拒算都在各片里跑（下面「分片」）；一片里一条都不用现跑就不起 cargo。拒算（单元闭包收不全）的那一条在片里照跑、不复用、不入库，成功句报拒算条数与头几处形态。
#   核账那一趟（合并那一步，含没有分片阶段时自己走的合并）不起 cargo：还有键对不上的就判红（crash_point_replay_package_pass_check）。
#   成功句首「按条复用」：现判几条、复用几条、拒算几条（scripts/gate/merge-and-verify.sh 读它）。
#   按条复用失灵（test-plan 退非 0、退 0 而没打 PLAN、它或 verdict-record 的复用哨兵报「复用失灵」「判定不稳」「强制现判没起作用」）：判法只在
#   scripts/gate/admission.py 的 plan-failure（文件头「按条复用失灵」），按条复用的几格把那一趟的输出交给它。crash-case-replay 格计划算不出（失灵与别的算不出）就不起 cargo、判红；
#   feature-gated-tests 格计划算不出照旧全部现判跑完、结论判红。
#   崩溃枚举用例的判定单位是崩溃状态块，块键与库写在 crates/singlefs-verification/src/crash_identity.rs 文件头；用例跑完打一行
#   CRASH_CASE_BLOCKS blocks=<块数> reused=<复用> fresh=<现判> red=<红> refused=<拒算>（接口原样写在 scripts/gate/admission.py 文件头「崩溃枚举用例的复用不在这里」）。
#   用例逐条按块跑（run_registered_crash_cases）：照登记表的次序，起 crash-case-command 交的命令（判定在 prune-server 里那一份库，引擎经客户端读写，不设库的变量）；
#   各片带 --share 只判领的块，核账那一趟整份、块照键复用、现判一块就判红；日志判法在 admission.py crash-case-judge（test result 恰好 1 passed、登记的计数行、线程），
#   块那一行在 admission.py crash-case-blocks（有红判红），都只在核账那一趟判。
#   登记行第三列带 red-blocks=judged-by-test 的用例（故障注入之后的红块由用例自己认下；admission.py crash-case-judged-by-test 答）：cargo test 退非 0 判红，退 0 的块那一行
#   带 --ran 只核在不在、数对不对得上（blocks = reused + fresh + refused），库里有红块也不判红；没登记的用例块那一行有红照旧判红。发现表同样只打印、不判红（red_states 不是 0、发现日志缺节都不判）。
#   登记行第三列带 findings=not-written-by-the-case 的用例（环境守卫清掉了门禁给的发现日志变量；admission.py crash-case-findings-not-written 答）：用例日志里有 CRASH_POINT_REPLAY_FINDINGS 行而发现日志是空的不判红；
#   发现日志写了节的照判。带 blocks-line=not-printed-by-the-case 的用例（枚举按设计 panic 在打块那一行之前；admission.py crash-case-blocks-not-printed 答）：块那一行带 --no-line-registered，一行都没有不判，
#   有的照判；判定是 cargo test 的退出码。这三个标记只放宽登记了的那一条，判法不认用例名。
#   登记行第三列带 separate-run=<理由> 的用例（拆出来单独验证：全域跑得久，不随不指定范围那一趟跑；admission.py crash-cases <根> --separate-run 答）：没有 --item 点名崩溃枚举用例的那一趟
#   （不带参数、只给 --check、只点 feature-gated 的项）读出用例清单之后就把它们去掉，逐条打一行「! 本次未跑（独立验证，separate-run=<理由>）：<用例>」、往 $GATE_NOT_RUN_FILE 记一行，
#   计划、各片、合并三步都走这一段（crash_case_rows 从这里起只剩留下的那几条，核账只核它们），分片的计划、判绿那一句与 all-green 写明去掉了几条；--item 点名了的照常跑，
#   --list-items 照旧列它们（读的是登记表）。它们仍是登记的崩溃枚举用例：按条复用那一趟照旧经 --exclude-registered 与 --skip 排掉它们的用例函数。
#   单交照「逐项点名」写：--check crash-case-replay --crash-cases-only --item crash-case:<用例名>。弄坏开关 separate-run-unrecognized 下不认这个标记、照旧全跑。
#   线程数 SINGLEFS_CRASH_POINT_REPLAY_THREADS 没设取每台默认重活线程数（.claude/heavy-thread-default 的登记值，admission.py 的 crash_case_worker_threads 经 registered_heavy_threads 取）；
#   只起 1 个线程怎么判红，判法在 admission.py crash-case-judge。
#   每条用例的块由各片分着判（下面「分片」），核账那一趟（合并那一步，含没有分片阶段时自己走的合并）照块判定库复用、一块不现判才算过。
#
# 分片（task-server/task_server_lib.py 文件头「分片」；文件头第二行「# task-server-shards: *」让服务把经它交的每一种跑法——不带参数、--check、--item——拆成计划、各片、合并）：
#   这一道照环境里的 TASK_SERVER_SHARD_PHASE 只做其中一步（没有这个变量的见下面「没有分片阶段」）；这几个 TASK_SERVER_SHARD_* 变量读完就从环境里清掉，不传给 cargo、准入模块与用例。认不出的一步退 2。
#   plan   参数照常认，开跑前的静态判据、RocksDB 预编库的取值与前提照常判；这一趟要跑 crash-case-replay 格的往计划目录写 N 片（plan.json 与 shards），
#          N 的上界是多机配置点名的卡数（开了 GPU 取 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED，没写取 GPU_CARDS_BY_PRIORITY；没开 GPU 取 HOSTS 的台数；读不出退 78），
#          --item 点名的崩溃枚举用例里有登记了 shards-at-least= 的（被判的根下 .claude/gate.d/stage-inputs.tsv 第三列，admission.py crash-case-shards-at-least <被判的根> 答最大的那个数，同 crash-cases 读的那一份），
#          上界取它与卡数的大者；那个子命令答不出退 2。
#          N 照这一趟要现判的量定（crash_point_replay_decide_shard_count），不超过上界：不点名用例的整轮取上界；点名了的，块数要录过才知道（计划这一步只编不跑、不录），
#          取每条点名的用例上一趟整份跑记下的块数（日志根 <git common-dir>/singlefs-crash-point-replay-logs/ 下有 whole-case-logs 记号的目录里最新的那一份，
#          crash_point_replay_recorded_case_blocks；记号由核账那一趟留，一片的日志只有它领的那一份的块数、不认），加起来除以一片担的块数
#          CRASH_POINT_REPLAY_BLOCKS_PER_SHARD；不带 --crash-cases-only 的另把下面算好的 test-plan 里要现判与拒算的测试目标数除以 CRASH_POINT_REPLAY_TARGETS_PER_SHARD；
#          都向上取整、取大的、至少 1。有一条用例的块数没有记录、或 test-plan 没算成，取上界。成功句「✓ 分片的计划：N 片（…）」写明取的是哪一个
#          （样本 shard-plan-small-named-case-one-shard、shard-plan-shards-at-least-sixteen、shard-plan-whole-run-takes-cards、shard-plan-unknown-blocks-takes-cards）。
#          计划这一步把这一趟登记的崩溃枚举用例要用的测试二进制编好一份放进计划目录下的 test-binaries/（crash_point_replay_prebuild_case_binaries）：
#          每条用例照 crash-case-command 交的命令（特性照登记与多机配置，开了 GPU 的带 gpu），同一组的测试目标并成一条 cargo test … --no-run（不持卡），
#          测试二进制硬链接进去（服务给计划那一步的编译目录在这一步收尾就删、派到第二台的在第二台本地盘上，各片读不到；计划目录在共享工作根下，两台同一个绝对路径；
#          不在同一个文件系统的拷一份：拷的去掉调试段，objcopy --strip-debug 直接写出去掉调试段的一份、原件不动，调试段占二进制的九成以上、判器核对读的键表在 .rodata 里，
#          objcopy 用不上整份拷；crash_amplification_place_binary_without_debug_info，按条复用那一趟的在 test-targets-parallel.py --prebuild-into 里同样处理），
#          连同判器核对要读的两张表照同样的相对位置放（同一份表只拷一份）；编不过、没打 Executable 行的那几个不放、打一行，各片自己经 cargo test 起。
#          不带 --crash-cases-only、下面那份 test-plan 放成了、里面有要现判或拒算的测试函数的，再把按条复用那一趟的测试二进制编好一份放进 test-binaries/package-pass/
#          （crash_point_replay_prebuild_package_pass_binaries：scripts/gate/test-targets-parallel.py --prebuild-into，参数与环境同各片起那一趟的，它写的清单 targets.tsv
#          是「目标 → 编好的二进制」；编不过只打一行，各片照旧经 cargo 编）。各片读了计划那一步的 test-plan（源码在计划之后没改）、清单在的，每个放进去的二进制照崩溃枚举用例那一路
#          核判器键所读的文件（crash_amplification_judging_inputs_of_binaries 一次核完：变了的那个目标改经 cargo，核不出的照用：源码没改已由读 test-plan 那一步核过；
#          别的片领的目标不核），把清单交给 test-targets-parallel.py --binaries，它直接起清单里的二进制、输出一件一份文件各收各的、开头照 cargo 写 Running 行（它文件头 ⑥），清单里没放的目标仍经 cargo；
#          别的片领的目标（计划里 judge、refused 行的目标根文件按 C 字节序排、序号模片数不等于片号的，写在 <计划输出>.other-shard-roots）这一片不起：交 --drop-sources，--skip 名单也只收
#          这一片起的目标的行带来的名字（admission.py plan-skip-names --except-target-roots-file；同名而要现判的照旧看全部目标；起测试的不是 test-targets-parallel.py 时都不收窄）；
#          所以两路都编好了的时候，各片与合并都没有会编的那一步（样本 package-pass-shards-run-the-plan-binaries-without-building）。
#          计划那一步放的测试二进制谁删（crash_point_replay_drop_shared_binaries，计划目录本身服务留着）：合并那一步收尾一律删；一片没判绿就收尾（判红、被停、出错）的删
#          （不会再有合并来用它，还没起的兄弟片读不到、退回自己编）；计划那一步没写成 plan.json 的删；判绿收尾的片不删。片被 KILL（撞上限）不走收尾，留给重排的那一次接着用。
#          这道门禁删不到的两种：整件被取消时一片都还没起（没有哪一步走到收尾）；片被 KILL 之后服务不再重排。这两种的兜底归服务：父任务（整件）有了结局时连计划目录下的
#          test-binaries/ 一起删（task-server/task_server.py 的 shard_binaries_removal：父任务有了结局与 task clean 时删，只删这一个子目录）。
#          各片与合并都不拿卡（文件头「显卡」：判段写待判行、等 GPU 批处理服务判）、都不钉台，哪台、内存额度由服务定，
#          每片的线程照服务交的每台默认重活线程数开满；
#          不跑那一格的（--list、--check 没点它）写 0 片，合并那一步照常跑点了的格；这时计划这一步不编、不起 cargo。
#          不带 --crash-cases-only 的另算一遍不带特性那一趟（按条复用）的 admission.py test-plan，连同它的输入（inputs：种类、根、请求、登记函数表）与算之前打的标记（stamp）
#          放进计划目录下的 test-plan/（plain.plan、plain.plan.out）：各片读它，不再各自起 test-plan（十几片同时读全部源码、第二台经共享盘读几个 GiB 会互相拖）。算不出只打一行、不判红。
#   shard  只做 crash-case-replay 格里起 cargo 的两样（判的都经客户端写进 prune-server 那一份判定库），别的格不跑：
#          按条复用那一趟：先读计划目录 test-plan/ 里计划那一步算好的计划（输入与此刻相同、计划之后验证包读的源码没有比标记新的文件才读），不再起 admission.py test-plan；
#            读不到、读不了、输入对不上、源码比标记新，打一行原因、这一片自己算。计划里要现判（judge）与拒算（refused）的测试函数按所在测试目标（目标根文件）分：目标按 C 字节序排好、序号模 N 等于片号的归这一片，
#            一个目标的测试函数全在同一片（整份现判的测试二进制照旧存下覆盖行）；别的片的那几条在计划里改记 registered、经 --exact --skip 排掉，判绿判红照「复用」一段记库；
#            跑完把这一趟的日志交 crash-flow-coverage.py collect 存覆盖行（存不成只打一行，合并那一步的覆盖格照「行不全」办）。
#          登记的崩溃枚举用例：逐条照 admission.py crash-case-command --share <片号>/<N> 起（SINGLEFS_CRASH_AMPLIFICATION_SHARE 设成这一份）；
#            计划那一步放下了这条用例的测试二进制（test-binaries/binaries.tsv 里包、特性、测试目标对得上）、文件在、判器键所读的文件此刻都没变的，不经 cargo、直接起它
#            （crash_point_replay_use_shared_case_binary，只录那一趟也是）；有一样不成打一行原因（「计划那一步没留下…」「…不在了」「…变了」「…核不出」）、这一片自己经 cargo test 编，
#            引擎只判这一片领的块、直接写进服务；这一步只看 cargo test 的退出码，日志与块那一行由核账那一趟判。
#          有红这一片退 1（服务撤掉还在排的兄弟片、不起合并，整件的结局就是这一片的）；被停照退 130 / 143；prune-server 不可用退 75。
#   merge  只核账：九格照常判，crash-case-replay 格不现判（服务交 TASK_SERVER_SHARD_PHASE=merge 的那一步，与没有分片阶段时自己走的最后一步，是同一段代码）：
#          按条复用：test-plan 照当时的源码现算、不读计划目录里的（打一行说明）；计划里还有键对不上（judge）的测试函数不起 cargo、判红（各片判过的按此刻的键在服务里读不到：源码中途变了、或片连的不是同一份服务）；
#            拒算的（不入库）算各片跑过（片都退 0 才起合并）；
#          崩溃枚举用例：照 crash-case-command 起整份（0/1）的核账那一趟（同各片：计划那一步放的测试二进制用得上就直接起，用不上自己经 cargo test 编），
#            日志目录里留一份 whole-case-logs 记号（下一趟计划那一步照它认整份的块数），判法同上面（crash-case-judge、块那一行、发现日志），另核块那一行的现判数是 0
#            （各片判的块按此刻的键全在服务里读到）；登记了 blocks-line=not-printed-by-the-case 的没有那一行、不核。现判数大于 0 判红。
#          服务交的 TASK_SERVER_SHARD_MISSING 不空（人手取消了一片）：一格都不判，退 77、点名缺的片。
#   没有分片阶段  环境里没有 TASK_SERVER_SHARD_PHASE（gate.sh 整轮、样本档与别的不经服务起的）：这一件里依次走计划、单片、合并三步，用上面 shard 与 merge 同一套代码，不另有一条不分片的路：
#          计划：这一趟要跑 crash-case-replay 格的定 1 片（第 0/1 片，SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/1），不跑那一格的 0 片、没有单片这一步；多机配置不读，不分第二台；
#            不预先编测试二进制（单片与合并在同一个进程里、用同一份编译目录）。样本根的 .gate-shard-count 点名 N 片的照服务分片走：依次走 N 片，计划这一步预先编测试二进制
#            放草稿目录里的 shard-plan/test-binaries/，各片与合并照它直接起（样本 shards-run-the-plan-binaries-without-building、shard-builds-itself-when-the-plan-binary-is-missing）；
#            不带 --crash-cases-only 的另算一遍不带特性那一趟的 test-plan 放草稿目录里的 shared-plan/，单片读它（test-plan 只在计划与合并各起一遍，单片不起）；
#          单片：照上面 shard 那一步做：键对不上的测试函数与库里没有的块都在这一步现判、写进 prune-server；有红退 1、prune-server 不可用退 75、被停照退，这三种都不起合并；
#            单片留下的按条复用那一趟的本次未跑记号交给合并那一步接着报，覆盖行没存成的记号清掉（合并那一步照键取覆盖行、自己再记）；
#          合并：照上面 merge 那一步做，单片判过的按此刻的键全在库里读到、现判 0 才过；单片现判过而按条记库不认的测试函数（打了 NOT_RUN_、结果行认不出结局）库里本来就没有键相同的判定，不算没人判。
#          弄坏开关 no-phase-runs-as-merge 下没有分片阶段不做计划与单片、直接当合并走：库里没有的块与键对不上的测试函数没人现判，核账判红。
#   日志：每趟一个新目录 <common-dir>/singlefs-crash-point-replay-logs/<开跑时刻>-<pid>/，跑完不删；log.<用例名> 是用例日志，同名加 .findings.tsv 是发现日志
#   （行格式以 crates/singlefs-verification/src/crash_point_replay_progress.rs 为准）。发现日志判红：某节没有 summary、red_states 不是 0、定稿行数与 signatures= 对不上，
#   或发现日志空而用例日志里有 CRASH_POINT_REPLAY_FINDINGS 行（放宽的两种标记见上面「复用」）。发现表超过 30 个签名只打前 30 行。
#   不指定范围（没给 --check，也没给 --item）而且没缩过域判绿的这一趟，在日志目录里留一份 all-green（只记这一趟绿过，scripts/gate/apply-writer-patch.py 数「上次崩溃验证全绿以来打了几个补丁」用；不是复用的标记）。
#   缩过域的那一趟（环境里设了 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 或 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO，引擎只往小缩、大于它的常量的不放宽；值是不是非负整数由调用方的入口核）判绿也不写 all-green，
#   成功句写「缩过域、不算门禁通过」并逐个写出设了的变量与值；点没点范围都一样。这一趟登记了 exhaustive= 的计数行 exhaustive=false 不判（判法在 admission.py crash-case-judge，读同一份环境），没缩域的那一趟照判。
#   没有用例级复用标记，不比树、不比 HEAD，不问 stage-must-run.sh 与 change-touches-crates.sh；stage-inputs.tsv 本阶段那一行只登记前提与入口，不进任何键。
#
# 用法（参数顺序不限）：
#   bash .claude/gate.d/crash-point-replay.sh [项目根]                          九格都跑；gate.sh 只传项目根；经 task-server/task submit 交的由服务照上面「分片」拆
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
#   --crash-cases-only         只与点名崩溃枚举用例的 --item 同给（上面「--crash-cases-only」）：crash-case-replay 格只走按块那一路、不起按条复用那一趟，
#                              别的格照常；没点崩溃枚举用例退 2。--list 与 --list-items 照收、不看它
# 退出码：
#   0   绿；带特性那一格有本次未跑时也退 0，逐条报「本次未跑」
#   1   红；有一格红就退 1；分片的一片有红
#   2   用法错：参数认不出，格名或项写错或缺（含点了 feature-gated 的项而 --check 里没有 feature-gated-tests、带 --crash-cases-only 而没点崩溃枚举用例），项目根进不去，
#       TASK_SERVER_SHARD_PHASE 认不出、分片那几步缺服务给的变量或片号对不上计划，
#       样本档标记目录里的假 cargo 或假 rustc 不在或不可执行，PATH 上的 cargo 没解到标记目录里那一份
#   3   样本档流程判绿：样本档没跑过真的崩溃点重放，不与真跑的绿混在一起
#   75  prune-server 不可用：几格的 test-plan 或 verdict-record 退 75（plan-failure 判不可用，scripts/gate/admission.py 文件头「按条复用失灵」），
#       或备覆盖行时 crash-flow-coverage.py collect 退 75（覆盖、ignore、0 次三格都记不可用），别的格没有红；
#       退 75 的那一格就此收尾（计划那一步退 75 的不起 cargo），别的格照跑；有一格真红就退 1，红的与不可用的两样都报（不是红，也不算通过），
#       出路 python3 prune-server/control check、报给主 agent
#   77  本次未跑：没有 验证包；合并那一步缺片（TASK_SERVER_SHARD_MISSING 不空）
#   78  准入与运行条件没满足（preflight）；计划那一步从多机配置读不出卡数与台数；
#       登记的崩溃枚举用例有观察者交回的盘取不到卡而记了本次未跑的（文件头「显卡」），别的格没有红、prune-server 也没有不可用（不写 all-green，样本档也退 78）
# 不用共用库 lib/stage-cells.sh 跑格：它的汇总把 0、77 之外的退出码并成 1（样本档要退 3）；只 source 它取「弄坏开关只在样本根上认」那一个函数，
# 环境里设了 *_BREAK 而根不是样本根（没有 .gate-cells / .gate-break-sample、或是本仓、--staged 的临时树）就退 2、什么都不跑（--list 与 --list-items 在那之前，不判）。
# 解析参数的判法与共用库同一套（写错退 2、列出格名）。
# 样本只跑点名的格（.claude/rules/verification.md「门禁的结构」，判据同共用库的 stage_cells_apply_sample_marker）：没给 --check、被判的根里有 .gate-cells
#   （一行一个格名，# 之后与空白不算）、这个根（pwd -P）不是本阶段所在的仓、也没设 GATE_STAGED_FROM 时，只跑文件里点名的格；点了没登记的格、一格都没点退 2。
#   本阶段所在的仓与 --staged 的临时树里有这个文件不认，打一行照跑九格。弄坏开关 sample-marker-ignored 下一律不认。
#
# 覆盖：两条流的崩溃点重放由登记的崩溃枚举用例跑，新池新建文件与 E142 产物逐字比对、「覆盖写、释放、回退与复用」固定脚本到 E 与闭式比对，都由用例自己断言；
#   只做过 mkfs 的池可写挂载再发第一个文件版本那条流与新池新建文件逐项相同，由 cargo test 里的用例钉住，不另枚举。
#   登记的用例不标 ignored，也不在按条复用那一趟里跑：逐条按块跑；用例的判别力在用例自己（计数断言与同文件的靶向阳性对照）。
#
# feature-gated-tests 格（C604（不开特性，崩溃放量与 GPU 对拍编进来零条））：特性从 验证包 tests/ 里每个测试目标（顶层 .rs；目录式的 main.rs 连同它 mod 声明的子模块）的 cfg 现读，不另抄名单；
#   挂特性的文件跑到加复用 0 条、每处 NOT_RUN_NO_GPU_ADAPTER（GPU 对拍没卡时打这一行后返回成功）都报「本次未跑」、不判红；一个挂特性的文件都没有，记无对象可判。
#   清掉调用方的 SINGLEFS_CRASH_AMPLIFICATION_* 与 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE：这一趟不写真的判定库、不按放量的规模跑。
#   键与跑的环境同一份：这一趟（与产品 GPU 真显卡那一趟）的 test-functions-request 也在清掉那些变量的环境里起（crash_point_replay_test_functions_request 用 env 带着 cargo 前面那一串 -u），
#   admission.py 的 DOMAIN_ENVIRONMENT_VARIABLES 按那一刻的环境把改判定域的变量写进键，清掉的变量在键里写成没设；不然键按没生效的变量算（门禁自己没清的环境），
#   不带特性那一趟按调用方的值记下的判绿，带特性那一趟（cargo 实际没看见这个变量）照键复用，跑的和记的不是同一个域。
#   特性里有 gpu 时这一趟不领卡（文件头「显卡」）：环境里有 GPU_CARDS_BY_PRIORITY 就原样交给 cargo 的 env，没有照起；要卡的测试读不到 GPU 服务的结论就红在测试里、这一格判红。
#   特性里有 gpu 时这一趟的测试参数带 `-- --nocapture`、不带 --test-threads：测试线程同特性里没有 gpu 的，照服务交下来的 RUST_TEST_THREADS（登记的每台默认重活线程数），
#   同一个测试二进制里并行的 GPU 用例经卡池排队（crates/singlefs-verification 的 gpu_test_start，.claude/rules/verification.md 里 GPU 那一条）。
#   FEATURE_PASS_LAST_TARGETS 里的测试目标（文件头「格的次序」）特性里有 gpu 时从这一趟拿出来，这一趟的判定先记（上面「格的次序」），再另起一趟 cargo test（同一张卡、同一组特性与跳过的测试函数）；
#   两趟的输出并成一份照读（计数、没卡没跑、判在 CPU、红的目标），逐条认结局两趟各认一遍，认出的行并进同一份清单。
#   样本 features-gpu-last-pass 钉这一条。
#   测试线程只管同时起几个测试，不管一个测试里 CPU 判器自己开的线程：GPU 对拍测试的 CPU 一侧只读 SINGLEFS_THREAD_CAP，没设取登记的每台默认重活线程数
#   （SINGLEFS_THREAD_CAP 设了照它，再没设取 .claude/heavy-thread-default 的登记值；取法在 crates/singlefs-verification/src/heavy_threads.rs 的 own_variable_or_default_heavy_threads）。样本里的假 cargo 把收到的参数打成一行 CRASH_POINT_REPLAY_PROGRESS sample-cargo-arguments: …，阶段转出来；features-supplement-not-run 的 expect 核这一行 --nocapture 后面紧接 --exact，features-gpu-last-pass 的假 cargo 另打 sample-cargo-test-threads: none，都不许有测试线程参数。
#   几条同时跑时 libtest 在测试跑完才打「test <名> ... <结局>」；别的测试打的字夹进起头与结局之间的，结局单独起一行接上照记，认不出的那几条不记（下一趟照跑），不算红；
#   这一趟的记号（NOT_RUN_、判段判在 CPU）照几条同时跑认：打这一行时还没出结果的都算（lib_libtest_result_lines.py 文件头「按条记库」）。
#
# product-gpu-real-card-tests 格（产品的后台 GPU 包的测试包 singlefs-background-gpu-tests 的真显卡对拍；欠账出处 records/2026-10-23-subagent拆分提案.md 第一百二十九节「产品 GPU 包的真卡对拍没有门禁在跑」）：
#   对拍测试目标靠特性 tests-on-a-real-gpu 才编（Cargo.toml 的 [[test]] required-features，或测试文件里 cfg 写了 feature = "tests-on-a-real-gpu"），
#   不开这个特性的 cargo test 整包与整工作区都不碰卡，所以 feature-gated-tests 格（只认 验证包 tests/ 里 cfg 点到的特性）认不到它们。
#   这一格按包与特性认（PRODUCT_GPU_PACKAGE_NAME、PRODUCT_GPU_FEATURE_NAME）、不按文件名写死：product_gpu_target_rows 从包目录现读哪几个测试目标挂这个特性，别的实现员以后再加的对拍测试目标也自动进来。
#   一个目标都没有（包不在、包有特性而没有目标挂它）记无对象可判（往 $GATE_NOT_RUN_FILE 记一行，不判红）；包在而 [features] 里没有这个特性判红（特性名对不上了，三处一起改：本阶段、
#   lib_project_code_runs.py 的 PRODUCT_GPU_FEATURE、crates-mutation-rows.py 的 GPU_TEST_FEATURES）。
#   起法：cargo test --release --no-fail-fast -p singlefs-background-gpu-tests --features tests-on-a-real-gpu --test <目标>… -- --nocapture（不带 --lib：库单测不上卡，归不带特性那一趟），
#   不领卡（文件头「显卡」，与 feature-gated-tests 带 gpu 特性那一趟同一套，crash_point_replay_pass_caller_cards）：环境里有 GPU_CARDS_BY_PRIORITY 就原样交给只判那一趟，没有照起；
#   要卡的对拍读不到 GPU 服务的结论就红在测试里、这一格判红。
#   两段：对拍用例的输入经判定库交接（测试包 tests/common_pairing_input_through_the_library：录入键、清单、按内容摘要存都是验证包 crash_library 那一份）。这一格同一条 cargo test 起两趟：
#   先只录（env 里 SINGLEFS_CRASH_AMPLIFICATION_STAGE=record、摘掉 GPU_CARDS_BY_PRIORITY，不持卡；每条要卡的用例把输入写进库、打一行 RECORDED_ONLY 就返回，日志 <包名>-real-card.record.log），
#   退非 0 判红、只判那一趟不起；录成了再只判（SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge：从库里读回输入对拍），逐条认结局、记库、数跑到的条数都只读只判那一趟的日志。
#   样本 product-gpu-record-red（只录那一趟退非 0：判红、只判那一趟没起）、product-gpu-card-missing（环境里没有卡：只录与只判两趟都是 cards=[unset]，判绿）钉这一条；
#   别的 product-gpu 样本的 expect 另核假 cargo 打的 sample-cargo-stage 两行（stage=record cards=[unset] 在前、stage=judge 带环境里那一串在后）。
#   对拍测试的 CPU 一侧只读 SINGLEFS_THREAD_CAP（没设取登记的每台默认重活线程数），测试线程只管同时起几个测试。
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
# 每一趟当场写结局（records/2026-10-29-subagent拆分提案.md 第一百七十节「门禁一格里前一趟的结局等整格收尾才写」：feature-gated-tests 格经内存包装起，
#   排在最后的那一趟撞顶被杀，外层日志只有两趟起跑的标题行，带特性那一趟绿没绿、红在哪一条没有一行，整份输出在草稿目录里跟着没了）：
#   一格里分几趟 cargo 的，每一趟 cargo 一收尾就由 report_pass_outcome 往标准输出打一行这一趟的结局，不等整格收尾：
#   「· <格名>：<哪一趟>收尾，当场写结局：cargo 退 <码>；test result 汇总行 <几> 行（合计通过 / 失败 / ignored；FAILED 的汇总行几行）；FAILED 的测试函数 <几> 条：<名字>…；整份输出：<路径>」。
#   打的趟：crash-case-replay 格不带特性那一趟、每条登记的崩溃枚举用例那一趟；feature-gated-tests 格带特性那一趟、排在最后另起的那一趟
#   （排在最后的那一趟起跑之前，带特性那一趟的结局已经打了）。
#   整份输出的路径：按条复用的几趟写在每一趟输出目录 ${TMPDIR:-/tmp}/crash-point-replay-passes.XXXXXX/ 里（草稿目录之外：被 TERM 停时 bash 照样走 EXIT trap 删草稿目录，
#   这一份不跟着删）；每条用例那一趟写在跑完不删的 <common-dir>/singlefs-crash-point-replay-logs/ 下。每一趟输出目录这一道判绿收尾（退 0、样本档退 3）时 EXIT trap 删掉，
#   别的（判红、被打断或被杀）留着、打一行路径，看完自己删。经 scripts/hosts/run-with-memory-cap.sh 起的，TMPDIR 是包装给的那一份：包装退非 0 时整份留着、
#   stderr 打出路径（它文件头「这一条命令自己的临时目录」），撞顶被杀之后照那一行找这一份。样本档收尾一律删掉。
#   弄坏开关 pass-outcome-at-cell-end 下等整格收尾才写（flush_deferred_pass_outcomes）：样本 killed-pass-keeps-earlier-outcome 判错。
# 进度行与逐提交的行：每一趟 cargo 的输出都接 crash_point_replay_pass_output（一小份 python，自己只读标准输入、写日志，不判）：
#   程序放在变量里、经 python3 -c 交：这一道被 TERM 停时 EXIT trap 马上删草稿目录，接输出的这一段可能还没读到程序；程序放草稿目录里就打不开，cargo 已打的行全丢。
#   进度行转到标准输出（前面加四个空格，机读的行都认行首，认不到它们）：一行里行首或空白之后有 <大写字母、数字、下划线>_PROGRESS 或 CRASH_AMPLIFICATION_BLOCK 的，名单不另抄，
#     CRASH_POINT_REPLAY_PROGRESS、CRASH_JUDGE_GPU_PAIRING_PROGRESS、CRASH_INJECTION_PROGRESS、FAULT_INJECTION_PROGRESS、BAD_DISK_INPUT_PROGRESS、ROOT_RING_READER_ADMISSION_PROGRESS 都在里面。
#   cargo 在打行而 5 分钟（.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」的进度间隔）没转出一行时，打一行
#     CRASH_POINT_REPLAY_PASS_ACTIVE seconds=<开跑多久> lines=<读了几行> bulk_lines=<多少行没进日志> last_line=<最后一行的前 200 字节>；这段时间 cargo 一行都没打就不打，
#     卡住的一趟日志照样不涨，统一任务服务照样判得出（task-server/task_server_lib.py 的 stalled）。
#     经 scripts/gate/test-targets-parallel.py 按测试目标并行起的两趟（文件头「按条复用两趟按测试目标并行起」）全部目标跑完才回读，跑的时候管道里只有它每 5 分钟至多一行的
#     TEST_TARGETS_PARALLEL_PROGRESS（这段时间有目标跑完或输出文件变长才打），名字带 _PROGRESS，照上面的认法转出来。
#   逐提交的行（行首是 GPU_LIFECYCLE 或 CRASH_JUDGE_ACCOUNTING_SLICE，GPU 判器每次提交、每片各打）不进日志，只留计数与末尾现场：量大，这两种行又没有哪个判法要读，所以滤在这里。
#     两种行在 crates 里的出处不同：GPU_LIFECYCLE 行由 gpu_lifecycle_line、gpu_lifecycle_trace、gpu_work_trace 三个单元打，它们登记成 GPU 判器那一组不沿它收的单元
#     （crates/singlefs-verification/src/judging_units_entries.rs 的 UNITS_THAT_ONLY_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW），改它们不换那一组的键；
#     CRASH_JUDGE_ACCOUNTING_SLICE 行由 GpuCrashJudge::judge_one_dispatch 打，它是 GPU 判器那一组的入口、没登记成不沿它收，改这一行的字就换那一组的键。
#     一行里夹着判法在行中间认的记号（NOT_RUN_，CRASH_ 起头的判段、覆盖行，I-<数> 计数）或打散进来的结果行、进度行、panic 的照进日志，计数里写「夹了别的记号」。
#     计数在 <日志>.bulk-lines.counts；末尾现场在 <日志>.bulk-lines 与 <日志>.bulk-lines.previous（每写满 2 万行换一代，留最后 2 万到 4 万行，一行一写，整棵被 KILL 也留得下）；
#     卡住或撞顶时看正在做哪一步就看这两份：它们跟日志放一起，判绿收尾才删（文件头「每一趟当场写结局」），判绿的崩溃枚举用例当场删。这一趟有这类行时打一行计数、路径与最后一行（report_bulk_lines）。
#   写进日志的其余字节与 tee 相同；还没收尾的半行，不会是逐提交的行的先进日志（「test <名> ... 」被别的测试打的字夹住时要到结局打出来才收尾，卡住时靠它看现场）。
#   收到 TERM、INT、HUP 不退，读到管道收尾才退：写的一方被停之后，到手的行照样落盘。
#   弄坏开关 only-replay-progress-forwarded、pass-activity-silent、bulk-lines-kept、bulk-lines-with-marks-dropped、bulk-tail-dropped 各弄坏一处，样本 progress-lines-and-bulk-lines 钉这一条；
#   样本里每一代的行数与多久报一行取标记目录里的 bulk-tail-lines、activity-seconds。
#
# 显卡（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：卡只归 GPU 批处理验证服务（gpu-verification-server/），它照多机配置开卡、一直持着、自己清场。
#   这一道不领卡、不挑卡、不清场，文件头不写要卡的声明；要上卡的三处照下面跑：
#   - 登记的崩溃枚举用例：带 gpu 特性的与别的一样，一条一趟、录入段与判段在同一趟里先后跑（admission.py crash-case-command 交的命令，不带 --stage）。
#      判段配了 GPU 时把这台机器领的要判的块写成判定库里的待判行、等 GPU 批处理服务判完（验证包 crash_amplification 的 judge_my_blocks_on_gpu），这一趟不持卡；服务读不了时判段一直等，卡住由任务服务判。
#      观察者交回的盘要在卡上判、取不到卡时验证包打 CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS 行、那一趟经 cargo test 退 101：
#      这一条不判红、记本次未跑（ITEM_DONE exit=78），出路是等 GPU 批处理服务收这一族待判行（任务不拿卡）；认法与出路只在共用库的
#      crash_amplification_failure_kind（observer-images-without-a-card 那一类）与 crash_amplification_observer_images_without_a_card_howto（crash_point_replay_case_has_no_card_for_observer_images）。
#      别的格与别的用例没有红时整道退 78（「退出码」）；有红照退 1，另报这几条本次未跑。分片的一片里同样记本次未跑、不算这一片红。
#   - product-gpu-real-card-tests 的对拍那一趟与 feature-gated-tests 带 gpu 特性那一趟（同入口 scripts/crash/crash-point-replay.sh 文件头「GPU」测试函数那一段）：
#      crash_point_replay_pass_caller_cards：环境里有 GPU_CARDS_BY_PRIORITY 就原样交给自己起的 cargo、打一行是哪一串；没有打一行、照起。
#      要卡的测试读不到 GPU 服务的结论就红在测试里，这一格照 cargo test 的退出码判红。带 gpu 特性那一趟仍是一整条 cargo test（不按目标并行）。
#   样本档里调用方交的卡只从 .crash-point-replay-sample-tools/caller-environment 来（进样本档先清掉调用方环境里的 GPU_CARDS_BY_PRIORITY）。
#   编译目录：照调用方环境里的 CARGO_TARGET_DIR 编，这一道不建、不删：文件头的会编声明「# task-server-compiles: yes」让统一任务服务给这一次尝试一份
#   （撞上限重来的那几次尝试接着用同一份、不重编，归服务删）；没给的用 cargo 的缺省。
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
#   带特性那一趟（feature-gated-tests 格）：特性里有 gpu 时不领卡（文件头「显卡」）；配置写了 ENABLE_GPU=1 而日志里有判段判在 CPU 的行
#     （同上两种）判红：配了 GPU、又编进了 gpu，判段必须在 GPU。
#   这两趟判段判在 CPU 判红时（cargo test 退 0），同一趟的判定照记（record_cpu_judging_pass，记法同 cargo test 退非 0 那一趟：红的与绿的交同一次 verdict-record）：
#     逐条认结局照「复用」那一段（开了 GPU，判在 CPU 的记号认作谁打的，那几条不记绿）；记号 complete、认作谁打的恰好一条的（打它时
#     只剩那一条还没出结果），归到那一条测试函数，照键记红、撤绿，下一趟现判；别的（几条同时在跑、这一段认不全）
#     只归到它所在的测试二进制，那个二进制里判绿的这一趟一条都不记；别的二进制判绿的照键记绿。
#     样本 gpu-configured-cpu-judged-records-the-rest（不带特性那一趟）、features-gpu-configured-cpu-judged-records-the-rest（带 gpu 特性那一趟）钉这一条。
#   两种记号在一行里哪里都认（前面是行首或空白，cpu_judging_lines）：测试打的一行可能接在别的测试的「test <名> ... 」同一行后面，
#     记号照几条同时跑认（同一行里的 test 名只是还没出结果的其中一条），只剩它一条还没出结果时归到它、照键记红；样本 features-gpu-configured-cpu-judged-on-test-line 钉这一条。
#   判得出的只有崩溃枚举那一类打的这两种行；给定状态与放量流水线的用例有的另写一份 ENABLE_GPU=0 的配置（GPU 对拍的 CPU 那一侧），它们的 mode= 不看。
#   配置没写 ENABLE_GPU=1（关着、没写、文件不在）两趟都不判判段在哪，判段判在 CPU 的那一条判绿照键记库（recordable 带 --cpu-judging-recordable）。
# 按条复用两趟按测试目标并行起（records/2026-10-23-subagent拆分提案.md 第九十七节 ①：cargo test 一个测试二进制接一个地跑，用例少的二进制跑的时候核空着，
#   看门狗报在用核数过少）：不带特性那一趟与特性里没有 gpu 的带特性那一趟，交给 scripts/gate/test-targets-parallel.py --jobs <每台默认重活线程数>
#   （scripts/hosts/heavy-threads.sh 的 heavy_thread_default，与 capped.sh default 取的是同一个数）：先 --no-run 一次编好，再按测试目标各起一条 cargo test、
#   同时至多那么多条；第二个 `--` 之后的 --nocapture 与按条复用的 --exact --skip <名>…
#   原样交给每一条（它文件头「测试二进制的参数」，自证「测试二进制的参数原样交给每个目标」那一格核）。各目标的输出按派活的次序回读进同一份日志，
#   每段照旧以 cargo 的「Running …」起头，下面按条记库、红的测试目标与 NOT_RUN_ 的读法不变；末尾多一行 TEST_TARGETS_PARALLEL …。
#   进度行要等全部目标跑完回读时才转出来（下面「进度行与逐提交的行」管不到的那一处）。
#   照旧一条 cargo test 的：样本档（假 cargo 多半不认 metadata 与 --no-run）、那份脚本或每台默认重活线程数取不到、带特性那一趟特性里有 gpu
#   （GPU 测试二进制一个接一个上同一张卡：卡池的显存账按进程记，几个进程同时上卡互相不排队，.claude/rules/verification.md 里 GPU 那一条）。
#   分片的一片交了计划那一步编好的清单（文件头「分片」）时样本档也经它起（带 --binaries）：样本自己的假 cargo 认 metadata 与 --no-run 的 JSON 产物；
#   样本档里它的峰值记录与用时表放这一趟的草稿目录（TEST_TARGETS_PARALLEL_PEAKS_DIRECTORY、TEST_TARGETS_PARALLEL_DURATIONS），不读写共享根里真跑记下的。
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
#   进样本档先核这一条（crash_point_replay_sample_service_check，调客户端的 selftest_service_problem，只看环境、不连服务；scripts/crash/crash-point-replay.sh 自证开跑先调的同一个）：
#   不在临时服务底下，判定库的键没有命名空间，样本读写的是真服务里与真门禁同一个种类的键（前一份样本或前一趟记下的绿被后一份复用、判错，样本的判定留在真库里），
#   退 2、一份样本都不判，拒绝说明给带包装的整条。样本根放 .sample-environment-bare 的，这一判按「环境里什么都没设」答（样本在临时服务底下被喂时造出裸跑的样子）；
#   样本 sample-root-bare-red 放它，期望退 2、点名没有临时 prune-server，其余样本都在临时服务底下被喂，是这一判的绿侧。
#   样本根的 .crash-point-replay-sample-tools/caller-environment（一行一个「名字=值」，# 起头的行不算）进样本档时当作调用方的环境导出：样本借它模拟调用方设了改判定域的变量。
#   标记目录里的 bulk-tail-lines、activity-seconds（一行一个数）改小末尾现场每一代的行数与多久报一行还有输出（文件头「进度行与逐提交的行」），不造几万行、不等 5 分钟。
#   标记目录里放一个 collect-unreachable（内容不读）：备覆盖行那一步的 collect 连一处连不上的服务（socket 不在、TCP 端口刚放掉、重连总时长 2 秒），
#     造「存取覆盖行时 prune-server 不可用」（collect_flow_pass）；按条复用的 test-plan / verdict-record 照常连服务。
#   标记目录里有 cargo-calls（setup.sh 建空的，假 cargo 每起一遍 test 追加一行收到的参数，假测试二进制转给它的那几遍不记）的，计划、各片、合并每一步结束打一行
#     「· 样本档：到<哪一步>结束为止 cargo test 起了 N 遍（其中只编不跑 M 遍）」（crash_point_replay_sample_cargo_count_note），样本据此钉计划编几回、各片与合并编没编。
#   样本根的 .gate-shard-phase（# 之后与空白不算，只认 plan）当作服务交的 TASK_SERVER_SHARD_PHASE=plan、计划目录取样本根下的 .gate-shard-plan-output/：
#     只演计划那一步（定片数、算 test-plan、预先编测试二进制、写 plan.json），写完计划退 3（样本档不退 0）；只在样本根上认，认法同 .gate-items。
#   样本根的 .gate-shard-count（一个 1 到 99 的整数，# 之后与空白不算）：没有分片阶段那一路照服务分片走这么多片、计划这一步预先编测试二进制（文件头「分片」没有分片阶段）。
#   判红时留下的日志（按条复用两趟 cargo test 退非 0 时那一趟的整份输出）拷到 ${TMPDIR:-/tmp} 下、出路指那一份，收尾删草稿目录时不跟着删；样本档里收尾
#   （crash_point_replay_exit_trap）另核每一份留下的在草稿目录删掉之后还在、内容与留下那一刻逐字节相同，核完删掉（样本不往 TMPDIR 里留东西）。
# 样本（fixtures/crash-point-replay.sh/，每份样本根的 .gate-cells 点名它算给哪一格，expect 的逐格那一行写到点了的每一格；红样本点了几格的，expect 另写一行「<格名>：红」，
#   code-tooling 的 fixture-claims 格照 .gate-cells 与这一行数红绿）：
#   没有分片阶段（样本档除样本根放了 .gate-shard-phase 的那几份都是）：点了 crash-case-replay 格的样本先走单片、再走合并。假 cargo 的 case_log 在有 SINGLEFS_CRASH_AMPLIFICATION_SHARE 的单片里原样打用例日志并留下 .judged-blocks/<测试目标> 记号，
#     没有这个变量的合并读到记号就把 CRASH_CASE_BLOCKS 那一行的 fresh 并进 reused、fresh 写 0（记号不在：库里没有，原样打）；
#     单片红（cargo 退非 0、被停、计划算不出、prune-server 不可用）整道退单片的码，后面的格与合并不跑，这几份样本的 expect 写单片那一步的句子（「第 0/1 片判红」）。
#   green、red                      登记的两条与三条崩溃枚举用例逐条按块跑：green 发现日志一节 0 个签名、一条不走发现日志，成功句报块现判 0、复用 3（单片那一步现判过），不指定范围判绿写 all-green；red 两个签名、没有 summary、32 个签名三条用例判红
#   static-check-red                一条调 enumerate_crash_states 却没登记 crash-case: 的 验证包用例、一份第一行没声明模块的测试文件：开跑前的静态判据判红、不起 cargo
#   sample-cargo-not-executable、rocksdb-entry-refused  假 cargo 不可执行退 2；验证包依赖 rocksdb 而假 cargo 不带 rocksdb-prebuilt.py 认的标记：起 cargo 之前判红退 1、假 cargo 一次都没起
#   registered-case-skipped-in-the-plain-pass  登记的 plain_stream_full 与放量分项 amplified_plain_stream_is_clean 在整包那一趟里经 --skip 排在最前面，按块那一路照跑，判绿
#   expand-up-to-narrowed-green     调用方环境里设了 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6（caller-environment）：按块那一路判绿，成功句写缩过域、不算门禁通过，不写 all-green
#   crash-case-provenance-green、crash-case-provenance-red  合表一项的崩溃枚举用例调 enumerate_crash_states：来历表有这一项的 enumeration_of_the_item 行判绿（样本档退 3），没有判红（退 1）
#   judged-by-test-green、judged-by-test-red、judged-by-test-missing-blocks-line  一条没带标记、一条登记 red-blocks=judged-by-test：带标记的 cargo test 退 0 且块那一行对得上判绿（红块不判）；退 101 判红；退 0 而没有块那一行判红
#   judged-by-test-red-in-the-merge  同一条登记了 red-blocks=judged-by-test 的用例：单片那一趟 cargo test 退 0，核账那一趟（假 cargo 的同名 .merge-exit 写 101）退 101：核账那一步判红、整道退 1（单片判红的样本不走到核账）
#   judged-by-test-with-red-blocks-green、judged-by-test-missing-blocks-line-after-the-run、unmarked-red-blocks  同一份判法的另三种：带标记的块那一行 red=2 照样判绿；缺块那一行判红；没带标记的块那一行 red=2 判红
#   以下三份样本钉合并那一步核账那一趟现判块数的许可：假 cargo 的核账那一趟照单片原样打块那一行（不把 fresh 并进 reused），一条 plain-stream 没带标记、一条 fault-injection-given-states 登记 red-blocks=judged-by-test
#   judged-by-test-fresh-equals-red-green  带标记的那条块那一行现判 2、红 2：现判不超过红块数，不算各片判的块读不到，判绿（样本档退 3）
#   judged-by-test-fresh-over-red-red  带标记的那条块那一行现判 3、红 2：多出来的一块算读不到，红在「核账那一趟现判了」
#   unmarked-fresh-after-shards-red  没带标记的那条块那一行现判 1、红 0：一块都不许现判，红在同一句；同一趟带标记的那条现判 2、红 2 照旧判绿
#   judged-by-test-findings-unjudged-green  两条都登记 red-blocks=judged-by-test：一条发现日志 red_states=9，一条用例日志有 CRASH_POINT_REPLAY_FINDINGS 行而发现日志缺节：发现表照打，两条都判绿
#   findings-judged-unmarked-red    两条都没带标记：一条用例日志有 CRASH_POINT_REPLAY_FINDINGS 行而发现日志缺节，一条发现日志 red_states=9：两条都判红
#   findings-not-written-by-the-case-green  一条登记 findings=not-written-by-the-case、用例日志有 CRASH_POINT_REPLAY_FINDINGS 行而发现日志空着判绿；同一趟没带标记的那条发现日志一节 0 个签名判绿
#   blocks-line-not-printed-green、blocks-line-not-printed-unmarked-red  日志里一行 CRASH_CASE_BLOCKS 都没有：登记 blocks-line=not-printed-by-the-case 的判绿（同一趟没带标记的那条块那一行照判）；没带标记的退 4 判红
#   separate-run-not-named-skipped-green  两条用例里 other-stream 登记 separate-run=：没点名的这一趟它不跑、报本次未跑，判绿那一句写去掉了 1 条，只判 plain-stream 那 2 块（样本档退 3）
#   separate-run-named-runs-green   同一份登记，.gate-items 点名 crash-case:other-stream：照常跑它、判绿（样本档退 3）
#   narrowed-not-exhaustive-green、not-narrowed-not-exhaustive-red  登记了 exhaustive= 的计数行 exhaustive=false：caller-environment 设了 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5 的判绿、成功句写缩过域、不算门禁通过；没设的判红
#   killed-pass-keeps-earlier-outcome  每一趟当场写结局：整包那一趟绿，登记的用例那一趟的假 cargo 给起它的阶段发 TERM（退 143），输出里已有整包那一趟的结局、没有「逐格」汇总
#   crash-audit-cells-green、crash-audit-caller-gpu-configuration-green  出口 4 那五格都判绿（覆盖行、计数行来自整包那一趟）；后一份另有 ENABLE_GPU=1 的调用方配置，判法相同
#   crash-audit-cells-red             五格各坏一样：StepKind 多一个成员、多一个按段登记的节点、一对新的评估 0 次、用例没有必红变异、旋钮出处不在：五格判红、整道退 1
#   crash-audit-case-log-green        覆盖行与计数行来自登记的用例的日志（extract），整包那一趟没有要判的测试：覆盖、ignore、0 次三格判绿
#   crash-audit-flow-records-unavailable、crash-audit-flow-records-unavailable-with-red  标记目录另放 collect-unreachable：覆盖、ignore、0 次三格记不可用、不判红，整道退 75；后一份另有一格真红，整道退 1，两样都报
#   gpu-configured-plain-pass-not-run、gpu-configured-plain-pass-judged-on-cpu  ENABLE_GPU=1，整包那一趟：用例打 NOT_RUN_ 记本次未跑；日志里有判段判在 CPU 的行判红
#   gpu-configured-cpu-judged-not-recorded、gpu-configured-cpu-judged-records-the-rest、cpu-judged-green-recorded  判段判在 CPU：开了 GPU 的判红且不记，同一趟别的判定照记；没开 GPU 的判绿照记
#   features-green、features-gpu-configured-green、features-gpu-last-pass  带特性那一趟：三个挂特性的文件都跑到了用例；ENABLE_GPU=1 时判段在 GPU；FEATURE_PASS_LAST_TARGETS 的目标另起一趟排在最后
#   features-gpu-not-run、features-zero、features-red、features-red-outside-the-tail、features-interrupted  没卡记本次未跑；跑到 0 条记本次未跑；带特性那一趟退 101 判红（红的目标在尾巴之外照样抄出）；被 TERM 打断判红
#   features-gpu-configured-judged-on-cpu、features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line  带 gpu 特性那一趟判段判在 CPU 判红，同一趟的判定照记（记号接在 test 行后面的也认）
#   features-shared-library、features-domain-variable-cleared、features-supplement-not-run、features-red-pass-records-green  两趟一份库；键与跑的环境同一份（清掉改判定域的变量）；补判不带特性那一趟打 NOT_RUN_ 的测试；有红的那一趟里判绿的照记
#   features-item-named-green、features-item-unknown  逐项点名（文件头「逐项点名」，样本根的 .gate-items）：点 feature-gated:store_only.rs 只编、只跑 tests/store_only.rs（不带 --lib，特性照整格那一组），
#                                    它的 ITEM_DONE 与 features-green 整格那一趟里同一项那一行逐字相同（点名单跑一项与它在整格里跑的判定相同）；点 tests/ 里没有的文件退 2、列出可点的项
#   red-outside-the-tail、red-pass-records-green、red-rejudged-next-pass  整包那一趟在单片那一步判红（红的目标在尾巴之外）；记红、撤绿，整道退 1、后面的格与合并不跑
#   red-record-failed-next-pass-rejudged  记红没成、下一趟不复用旧绿：上一版照判绿记进 shared_check 同一把键的绿；.unit-verdict-sample/red-over-green 让不带特性那一趟
#                                    的计划不读库（ADMISSION_BREAK=verdict-lookup-skipped）、上一版只在这一趟记一次，shared_check 现判红；.unit-verdict-sample/red-record-failure
#                                    让 admission.py verdict-record 记红没成、退 1，plan-failure 带 --red 判失灵、撤掉那条绿（「同一把键原有的绿已撤掉 1 条」）；
#                                    单片那一步判红、记红没成、撤掉那条绿，整道在那里退 1
#   unit-key-samples-green、unit-key-samples-red  按条复用（对照样本 S17、N19、F1、H2、H3）：要现判的恰好是 test_t、test_u、f1_middle、h2_test，复用的是 test_v、f1_first、f1_last、h3_test，假 cargo 照收到的 --skip 只打要现判的结果行；后一份让 test_t 判 FAILED
#   unit-key-plan-exit-red、unit-key-plan-sentinel-red、unit-key-plan-unavailable、unit-key-plan-unavailable-with-red  按条复用失灵：test-plan 退 1、哨兵报复用失灵：单片那一步判红、不起 cargo、整道退 1；退 75 判不可用、整道退 75（unit-key-plan-unavailable-with-red 另点了 crash-knobs，单片那一步就退 75、只读树的格不跑）
#   unit-key-gpu-switch-not-in-key、unit-key-not-run-withheld-per-test  GPU 开没开不进键；按测试函数不记（打了 NOT_RUN_ 的那一条、结果行被打散认不出结局的那几条）
#   product-gpu-green、product-gpu-red、product-gpu-no-card、product-gpu-card-missing、product-gpu-zero-tests、product-gpu-no-target、product-gpu-feature-missing、product-gpu-reuse-same-inputs、product-gpu-reuse-unrelated-change、product-gpu-reuse-unit-change
#                                    产品 GPU 包那一格的样本：三条对拍判绿并记库；判红；没卡逐条记本次未跑；环境里没有卡（只录与只判两趟都不带卡）判绿；跑到 0 条；没有目标挂特性；包里没有这个特性判红；复用三问
#   product-gpu-record-red  两段（文件头「product-gpu-real-card-tests 格」两段那一段）：只录那一趟退 101（cases/record.exit），这一格判红、只判那一趟没起
#   crash-cases-only-named-green、crash-cases-only-without-item  --crash-cases-only（样本根的 .gate-crash-cases-only）：登记两条用例、.gate-items 点 crash-case:plain-stream，
#                                    按块那一路只起它、判绿（样本档退 3），按条复用那一趟不起（样本里有三条要现判的测试函数，假 cargo 照 cases/default-pass.exit 退 101：起了就判红）；
#                                    不点用例退 2（没点的 other-stream 没有 cases/ 日志：照收而跑起来就判红）
#   progress-lines-and-bulk-lines     不带特性那一趟：六种进度行转出来，有输出而没进度行时报一行，逐提交的行只留计数与末尾现场（每一代 2 行，留 3 行），夹着 NOT_RUN_ 的那一行与打散的结果行照认：判绿记库 1 条、报本次未跑 1 处
#   plain-pass-red-crash-cases-still-run、plain-pass-red-and-crash-case-red-both-reported  不带特性那一趟红（测试二进制死于 SIGSEGV，不算被停）：单片那一步里登记的用例照跑、判完它们领的块，单片退 1；用例也红：两部分都报
#   plain-pass-stopped-skips-the-rest、plain-pass-test-binary-terminated-skips-the-rest  不带特性那一趟被停（cargo 退 143；cargo 退 101 而测试二进制死于 SIGTERM）：单片那一步里用例不接着跑，整道退 1、后面的格不跑
#   crash-case-stopped-skips-the-rest  登记的 plain-stream 那条用例被停：排在它后面的 other-stream 不接着起（cases/ 里没有它的日志：起了就判红），单片退 1
#   crash-case-no-card-for-observer-images-not-run  登记的 plain-stream 那条用例的日志里有观察者交回的盘取不到卡那一行（接在「test … ... 」后面）、cargo 退 101：
#                                    单片与合并都记本次未跑、不判红，ITEM_DONE exit=78，整道退 78、出路是等 GPU 批处理服务收这一族待判行
#   last-pass-runs-after-the-other-cells  排在最后的那一趟的假 cargo 把阶段停了（退 143）：别的格的结局与带特性那一趟的判绿记库都已经在输出里
#   features-main-pass-stopped-skips-last-pass  带特性那一趟被停（cargo 退 143）：排在最后的那一趟不接着起，这一格判红
#   no-phase-plan-shard-merge-green  没有 TASK_SERVER_SHARD_PHASE：自己走计划、单片、合并：登记的一条用例的块库里没有（假 cargo 单片那一趟打 fresh=2、留下记号），在单片那一步现判，合并那一步（记号在：块都读得到）现判 0、判绿（样本档退 3）
#   no-phase-test-plan-once-green  同上一份的输入，标记目录里再放一份假 python3（admission.py test-plan 每起一遍往 test-plan-calls 追加一行）：计划那一步算一遍 test-plan，单片读它（不再起），合并现算一遍：
#                                  三步结束时打的累计遍数是 1、1、2（样本档退 3）
#   shard-plan-small-named-case-one-shard、shard-plan-shards-at-least-sixteen、shard-plan-whole-run-takes-cards、shard-plan-unknown-blocks-takes-cards
#                                  计划那一步（样本根的 .gate-shard-phase；多机配置没开 GPU、HOSTS 5 台；setup.sh 照标记目录的 recorded-blocks.tsv 在日志根下造一个有 whole-case-logs 记号的目录）：
#                                  点名一条上一趟整份记下 8 块的用例写 1 片；点名一条登记了 shards-at-least=16、上一趟记下 4096 块的用例写 16 片（量要 64 片，不超过上界）；
#                                  不点名用例的整轮（两条都只记下 8 块）写 5 片；点名一条只有一片领的那一份块数（shard-share-blocks.tsv：没有记号的更新的目录，不认）的用例写 5 片。
#                                  四份都预先编了要用的测试二进制（假 cargo 的 --no-run 打 Executable 行点名标记目录 binaries/ 里的假测试二进制），写完计划退 3
#   shards-run-the-plan-binaries-without-building、shard-builds-itself-when-the-plan-binary-is-missing  没有分片阶段那一路照服务分片走（样本根的 .gate-shard-count）、--crash-cases-only：
#                                  两片：计划那一步编一回（cargo-calls 1 行、只编不跑 1 遍），两片与合并都直接起放进去的假测试二进制（判器键所读的文件都没变），到合并结束 cargo test 仍是 1 遍，判绿（样本档退 3）；
#                                  一片、标记目录里没有假测试二进制（只编那一步的日志里没有 Executable 行）：那一片与合并各打一行「计划那一步没留下它的测试二进制…自己经 cargo test 编」、各编一回（1、2、3 遍），判绿（样本档退 3）
#   package-pass-shards-run-the-plan-binaries-without-building  没有分片阶段那一路照服务分片走两片（.gate-shard-count）、不带 --crash-cases-only：假 cargo 认 metadata 与
#                                  --no-run --message-format=json（产物指到标记目录 binaries/ 里的假测试二进制，库单测那一个叫 singlefs_verification），计划那一步编两回
#                                  （崩溃枚举用例一回、按条复用那一趟经 test-targets-parallel.py --prebuild-into 一回），两片的按条复用那一趟带 --binaries 直接起放进去的假二进制
#                                  （src/lib.rs 归第 0 片、tests/plain_stream.rs 归第 1 片：各片只核、只起归它的那一个，另一个这一片不起、不核，--skip 名单只收归它的目标的），登记的那条用例也直接起；
#                                  到计划、两片、合并结束 cargo test 都是 2 遍、只编不跑 2 遍，判绿（样本档退 3）
#   case-binaries-copied-without-debug-info  同 shards-run-the-plan-binaries-without-building（--crash-cases-only、两片），标记目录里另有假 ln（一律退 1：硬链接放不成）与假 objcopy：
#                                  计划那一步放登记的两个用例的测试二进制都是拷的（硬链接 0 份、拷 2 份）、都去了调试段（2 份）、判器核对的表只拷 2 份（键表与摘要表各一份），各片与合并照样认得出键表、直接起，判绿（样本档退 3）
#   四份 features-* 的 tests/ 里挂特性的写法各有一处：整份 #![cfg(feature)]、折成几行的 #![cfg(all(…))]、逐条 #[cfg(feature)]，
#   与一个只在 // 注释里出现的特性名（读进来就找不到那组特性的日志）。样本里的特性 sample-store、sample-accelerator 是造出来的（验证包只有 gpu 一个特性）。
# 函数分在 .claude/gate.d/lib/ 下的四份库里，入口只留文件头、格名表、参数解析与按格跑的主流程；各份放的是：
# 库 lib/crash-point-replay-selection-and-plan.sh：格名表的打印与点名、测试函数计划的请求与判定种类、分片计划文件、样本档的标记与服务检查、按条复用计划的读写与计数说明
# 库 lib/crash-point-replay-package-pass-verdicts.sh：整包那几趟的测试二进制预编与收尾、分片数、崩溃枚举用例的测试二进制、已记判定的预取、判定的记绿记红、没跑的补报与每一趟结局的报告
# 库 lib/crash-point-replay-crash-case-cells.sh：被停的认法、跑测试的命令、整包按条复用那一趟的判、登记的崩溃枚举用例逐条按崩溃状态块那一路的跑与报（crash-case-replay 格）
# 库 lib/crash-point-replay-feature-card-and-audit-cells.sh：带特性那一趟（feature-gated-tests 格）、产品 GPU 真显卡那一格（product-gpu-real-card-tests）、崩溃流覆盖与只读树那几格、分片那一步与各格的起
# 弄坏开关里读开关的代码，一部分在 lib/ 下的库里；下面每条 GATE_CRASH_POINT_REPLAY_BREAK 开关的值，在哪份库的代码里被读，按库列在这里：
# 库 lib/crash-point-replay-selection-and-plan.sh：shard-plans-itself、merge-reads-the-shared-plan、registered-rows-in-plan、setting-file-whole-closure
# 库 lib/crash-point-replay-selection-and-plan.sh：directory-target-listing-unseen、gpu-switch-in-key、library-per-pass、sample-marker-ignored
# 库 lib/crash-point-replay-selection-and-plan.sh：test-functions-request-gate-environment、crash-cases-only-ignored、pass-output-program-in-scratch
# 库 lib/crash-point-replay-selection-and-plan.sh：sample-service-unchecked
# 库 lib/crash-point-replay-package-pass-verdicts.sh：shard-count-from-cards-only、shards-build-themselves、package-pass-shards-build-themselves
# 库 lib/crash-point-replay-package-pass-verdicts.sh：package-pass-other-shards-targets-run、case-tables-copied-per-binary、registered-cases-not-skipped
# 库 lib/crash-point-replay-package-pass-verdicts.sh：registered-rows-in-plan、setting-file-whole-closure、plan-unavailable-as-failure
# 库 lib/crash-point-replay-package-pass-verdicts.sh：plan-unavailable-exits-at-once、red-record-failure-kept-green、red-tail-only
# 库 lib/crash-point-replay-package-pass-verdicts.sh：pass-outcome-at-cell-end、withhold-whole-binary、cpu-judging-withheld
# 库 lib/crash-point-replay-package-pass-verdicts.sh：cpu-judging-recorded-when-gpu-configured、red-not-recorded
# 库 lib/crash-point-replay-crash-case-cells.sh：shards-build-themselves、package-pass-other-shards-targets-run、amplification-items-not-skipped
# 库 lib/crash-point-replay-crash-case-cells.sh：no-findings-file、ignore-unfinished、ignore-red-states、no-truncation、sample-caller-configuration-kept
# 库 lib/crash-point-replay-crash-case-cells.sh：cpu-judging-accepted、plain-pass-not-run-unreported、judged-by-test-ignores-exit
# 库 lib/crash-point-replay-crash-case-cells.sh：judged-by-test-unrecognized、ran-for-every-case、judged-by-test-fresh-unallowed
# 库 lib/crash-point-replay-crash-case-cells.sh：judged-by-test-findings-judged、findings-unjudged-for-every-case、findings-not-written-unrecognized
# 库 lib/crash-point-replay-crash-case-cells.sh：findings-not-written-for-every-case、blocks-line-not-printed-unrecognized
# 库 lib/crash-point-replay-crash-case-cells.sh：blocks-line-optional-for-every-case、plain-pass-red-withholds-green
# 库 lib/crash-point-replay-crash-case-cells.sh：plain-pass-cpu-judging-records-nothing、item-done-unreported、no-card-for-observer-images-as-red
# 库 lib/crash-point-replay-crash-case-cells.sh：package-pass-red-skips-cases、stopped-pass-runs-the-rest
# 库 lib/crash-point-replay-feature-card-and-audit-cells.sh：provenance-judge-not-run、no-feature-pass、features-not-from-source、ignore-feature-pass-exit
# 库 lib/crash-point-replay-feature-card-and-audit-cells.sh：ignore-zero-feature-files、directory-target-count-by-basename、ignore-gpu-not-run
# 库 lib/crash-point-replay-feature-card-and-audit-cells.sh：last-pass-merged、gpu-test-threads-one、product-gpu-cell-skipped
# 库 lib/crash-point-replay-feature-card-and-audit-cells.sh：product-gpu-test-threads-one、product-gpu-exit-ignored、product-gpu-not-run-ignored
# 库 lib/crash-point-replay-feature-card-and-audit-cells.sh：product-gpu-zero-ignored、product-gpu-targets-by-file-name
# 库 lib/crash-point-replay-feature-card-and-audit-cells.sh：product-gpu-record-holds-the-card、product-gpu-record-exit-ignored
# 库 lib/crash-point-replay-feature-card-and-audit-cells.sh：product-gpu-judge-without-the-stage、service-card-not-forwarded、stage-quiesces-by-itself
# 库 lib/crash-point-replay-feature-card-and-audit-cells.sh：cpu-judging-accepted、no-supplement、flow-records-unavailable-as-failure
# 库 lib/crash-point-replay-feature-card-and-audit-cells.sh：feature-pass-red-withholds-green、product-gpu-red-withholds-green
# 库 lib/crash-point-replay-feature-card-and-audit-cells.sh：feature-pass-cpu-judging-records-nothing、cpu-judging-line-start-only、feature-items-ignored
# 库 lib/crash-point-replay-feature-card-and-audit-cells.sh：feature-item-features-narrowed、item-done-unreported
# 库 lib/crash-point-replay-feature-card-and-audit-cells.sh：feature-main-pass-recorded-with-last-pass
# 只在这一份入口里读的：no-phase-runs-as-merge、all-green-not-written、narrowed-run-writes-all-green、static-check-skipped、directory-target-running-line-unseen
# 只在这一份入口里读的：directory-target-name-unstripped、sample-tools-unchecked、rocksdb-entry-unchecked、list-items-refusals-counted-only
# 只在这一份入口里读的：separate-run-unrecognized、crash-audit-red-ignored、feature-item-unknown-accepted、crash-cases-only-items-unchecked
# 只在这一份入口里读的：only-replay-progress-forwarded、pass-activity-silent、bulk-lines-kept、bulk-lines-with-marks-dropped、bulk-tail-dropped
# 只在这一份入口里读的：last-pass-before-the-other-cells
# 弄坏开关（只给证红用）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/crash-point-replay.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   GATE_CRASH_POINT_REPLAY_BREAK=no-phase-runs-as-merge  没有 TASK_SERVER_SHARD_PHASE 时不做计划与单片、直接当合并走（库里没有的块与键对不上的测试函数没人现判，核账判红）；点了 crash-case-replay 格的样本多数判错，下面逐份点名
#       no-phase-plan-shard-merge-green（期望退出 3，实测 1）；
#       blocks-line-not-printed-green、case-binaries-copied-without-debug-info、cpu-judged-green-recorded、crash-audit-caller-gpu-configuration-green、
#       crash-audit-cells-green、crash-audit-cells-red、crash-audit-flow-records-unavailable、crash-audit-flow-records-unavailable-with-red、
#       crash-case-no-card-for-observer-images-not-run、crash-case-stopped-skips-the-rest、expand-up-to-narrowed-green、
#       features-domain-variable-cleared、features-gpu-configured-cpu-judged-on-test-line、features-gpu-configured-cpu-judged-records-the-rest、
#       features-gpu-configured-green、features-gpu-configured-judged-on-cpu、features-gpu-last-pass、features-gpu-not-run、features-green、features-interrupted、
#       features-red、features-red-outside-the-tail、features-red-pass-records-green、features-shared-library、features-supplement-not-run、features-zero、
#       findings-not-written-by-the-case-green、gpu-configured-cpu-judged-not-recorded、gpu-configured-cpu-judged-records-the-rest、
#       gpu-configured-plain-pass-judged-on-cpu、gpu-configured-plain-pass-not-run、green、judged-by-test-findings-unjudged-green、judged-by-test-green、
#       judged-by-test-red、judged-by-test-red-in-the-merge、judged-by-test-with-red-blocks-green、killed-pass-keeps-earlier-outcome、
#       last-pass-runs-after-the-other-cells、no-phase-test-plan-once-green、package-pass-shards-run-the-plan-binaries-without-building、
#       plain-pass-red-and-crash-case-red-both-reported、plain-pass-red-crash-cases-still-run、plain-pass-stopped-skips-the-rest、
#       plain-pass-test-binary-terminated-skips-the-rest、product-gpu-card-missing、product-gpu-feature-missing、product-gpu-green、product-gpu-no-card、
#       product-gpu-no-target、product-gpu-record-red、product-gpu-red、product-gpu-reuse-same-inputs、product-gpu-reuse-unit-change、
#       product-gpu-reuse-unrelated-change、product-gpu-zero-tests、progress-lines-and-bulk-lines、red-outside-the-tail、red-pass-records-green、
#       red-record-failed-next-pass-rejudged、red-rejudged-next-pass、registered-case-skipped-in-the-plain-pass、separate-run-named-runs-green、
#       separate-run-not-named-skipped-green、shard-builds-itself-when-the-plan-binary-is-missing、shards-run-the-plan-binaries-without-building、
#       unit-key-gpu-switch-not-in-key、unit-key-not-run-withheld-per-test、unit-key-plan-exit-red、unit-key-plan-sentinel-red、unit-key-plan-unavailable、
#       unit-key-plan-unavailable-with-red、unit-key-samples-green、unit-key-samples-red、unmarked-red-blocks（没有单片那一步，库里没有块与键，合并核账判红）
#   GATE_CRASH_POINT_REPLAY_BREAK=shard-plans-itself  分片的一片不读计划目录里的 test-plan，仍各自起一遍 admission.py test-plan
#       no-phase-test-plan-once-green（输出里找不到「· 分片：第 0/1 片读计划那一步算好的 test-plan」）、
#       package-pass-shards-run-the-plan-binaries-without-building（输出里找不到「· 分片：第 0/2 片读计划那一步算好的 test-plan」：没读计划的片不用计划那一步编好的清单）
#   GATE_CRASH_POINT_REPLAY_BREAK=merge-reads-the-shared-plan  合并那一步也先读计划目录里的 test-plan（核账要的是此刻的键，不该读计划时算的）
#       no-phase-test-plan-once-green（输出里找不到「· 合并：不带特性那一趟的 test-plan 照当时的源码现算」）；
#       crash-audit-caller-gpu-configuration-green、crash-audit-cells-green（输出里找不到「覆盖行与计数行：不带特性那一趟：2 个测试二进制，整份现判 0 个」：合并读的是计划那一刻的 test-plan，要现判的 2 个还没判，整份现判的数按那一刻的，取回存着的行是 0 个）；
#       crash-audit-cells-red、crash-audit-flow-records-unavailable、crash-audit-flow-records-unavailable-with-red（同一处：覆盖行与计数行按计划那一刻的 test-plan 取）
#   GATE_CRASH_POINT_REPLAY_BREAK=shard-count-from-cards-only  计划那一步的片数一律取上界（卡数与 shards-at-least 的大者），不看这一趟要现判的量
#       shard-plan-small-named-case-one-shard（输出里找不到「✓ 分片的计划：1 片（取这一趟要现判的量算出的 1 片：点名的 1 条用例上一趟整份记下的块共 8 个（一片担 64 个）；上界 5 片（多机配置 HOSTS 点名 5 项，一项一片）；1 条点名的崩溃枚举用例（0 是全部登记的））」：写的是 5 片）、
#       shard-plan-shards-at-least-sixteen、shard-plan-whole-run-takes-cards、shard-plan-unknown-blocks-takes-cards（片数照旧是 16、5、5，成功句换成「…；弄坏开关 shard-count-from-cards-only 下不看这一趟要现判的量」，找不到各自 expect 里「✓ 分片的计划：」那一句）
#   GATE_CRASH_POINT_REPLAY_BREAK=shards-build-themselves  计划那一步不预先编测试二进制，各片与合并各自经 cargo test 编（crash_point_replay_prebuild_case_binaries、crash_point_replay_use_shared_case_binary）
#       shards-run-the-plan-binaries-without-building（输出里找不到「✓ 计划：登记的崩溃枚举用例要用的测试二进制按命令分 1 组各编一回（cargo test … --no-run，不持卡），2 个放进」）、
#       case-binaries-copied-without-debug-info（同上一份那一句）、
#       shard-builds-itself-when-the-plan-binary-is-missing（输出里找不到「CRASH_POINT_REPLAY_PROGRESS sample-cargo-build: target=plain_stream」）、
#       shard-plan-small-named-case-one-shard、shard-plan-shards-at-least-sixteen、shard-plan-whole-run-takes-cards、shard-plan-unknown-blocks-takes-cards、
#       package-pass-shards-run-the-plan-binaries-without-building（输出里找不到「✓ 计划：登记的崩溃枚举用例要用的测试二进制按命令分 1 组各编一回（cargo test … --no-run，不持卡），1 个放进」）
#   GATE_CRASH_POINT_REPLAY_BREAK=package-pass-shards-build-themselves  计划那一步不预先编按条复用那一趟的测试二进制，各片照旧经 cargo 编（crash_point_replay_prebuild_package_pass_binaries）
#       package-pass-shards-run-the-plan-binaries-without-building（输出里找不到「✓ 计划：按条复用那一趟的测试二进制编了一回」）、
#       shard-plan-whole-run-takes-cards（输出里找不到「· 计划：预先编按条复用那一趟的测试二进制（不持卡，只编不跑）：」）
#   GATE_CRASH_POINT_REPLAY_BREAK=package-pass-other-shards-targets-run  分片的一片也核、也起别的片领的测试目标，--skip 名单不收窄（crash_point_replay_settle_package_pass_binaries、run_checker_package_tests）
#       package-pass-shards-run-the-plan-binaries-without-building（输出里找不到「：判器键所读的文件没变 1 个、核不出照用 0 个（源码在计划之后没改）、变了改经 cargo 0 个、计划那一步没放的 0 个经 cargo；别的片领的 1 个测试目标这一片不起、不核」）
#   GATE_CRASH_POINT_REPLAY_BREAK=case-tables-copied-per-binary  计划那一步放登记的崩溃枚举用例的测试二进制时，判器核对要读的表为每个二进制各拷一遍（crash_point_replay_prebuild_case_binaries）
#       shards-run-the-plan-binaries-without-building（输出里找不到「（拷的里去掉调试段 0 份；判器核对的表拷 2 份）」：拷了 4 份）、
#       case-binaries-copied-without-debug-info（输出里找不到「（拷的里去掉调试段 2 份；判器核对的表拷 2 份）」：拷了 4 份）
#   GATE_CRASH_POINT_REPLAY_BREAK=registered-cases-not-skipped  整包那几趟不把登记的崩溃枚举用例函数排进 --skip（计划里的行照去）
#       registered-case-skipped-in-the-plain-pass（输出里找不到「sample-cargo-arguments: … --exact --skip amplified_plain_stream_is_clean --skip plain_stream_full」）
#   GATE_CRASH_POINT_REPLAY_BREAK=amplification-items-not-skipped  不取放量入口 ITEMS 里的用例函数：整包那几趟不排放量分项
#       registered-case-skipped-in-the-plain-pass（输出里找不到「--skip amplified_plain_stream_is_clean」那一行）
#   GATE_CRASH_POINT_REPLAY_BREAK=all-green-not-written  不指定范围判绿也不写 all-green
#       green、no-phase-plan-shard-merge-green、no-phase-test-plan-once-green（输出里找不到「不指定范围，这一趟判绿：」）
#   GATE_CRASH_POINT_REPLAY_BREAK=narrowed-run-writes-all-green  缩过域（环境里设了展开深度变量）的那一趟判绿也写 all-green、不写缩过域那一句（不看缩域变量）
#       expand-up-to-narrowed-green（输出里找不到「缩过域、不算门禁通过：SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6」）；
#       narrowed-not-exhaustive-green（输出里找不到「缩过域、不算门禁通过：SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5」）
#   GATE_CRASH_POINT_REPLAY_BREAK=provenance-judge-not-run  crash-case-provenance 格不起 crash-case-check.py 的 provenance 判法、直接记绿
#       crash-case-provenance-red（期望退出 1，实测 3：来历表缺行的合表项没人判，这一格记绿）
#   GATE_CRASH_POINT_REPLAY_BREAK=registered-rows-in-plan  整包那几趟与样本档的上一版算计划时不带 --exclude-registered（登记的崩溃枚举用例进计划、能被挑成复用哨兵、样本档替它记绿）
#       unit-key-samples-green（输出里找不到「上一版（.unit-verdict-sample/prior）的 8 条测试函数」：plain_stream_full 也记了）、
#       unit-key-gpu-switch-not-in-key（输出里找不到「上一版（.unit-verdict-sample/prior）的 3 条测试函数」）、
#       unit-key-plan-sentinel-red（输出里找不到「复用失灵：crates/singlefs-verification/tests/setting.rs::test_t 键相同却现判了」：头一条请求成了 plain_stream.rs 那一条）
#   GATE_CRASH_POINT_REPLAY_BREAK=setting-file-whole-closure  整包那几趟每条测试函数的键按它所在的整份设点文件算（文件里每个单元都进键：改同文件里别的测试也作废）
#       unit-key-samples-green（输出里找不到「复用 crates/singlefs-verification/tests/setting.rs::test_v」：N19 那一格，只改了 test_u 的一个字，同文件的 test_v 也被判成要现判）；
#       unit-key-samples-red（输出里找不到「--skip f1_first --skip f1_last --skip h3_test --skip test_v」：复用的那几条不再 --skip）；
#       red-record-failed-next-pass-rejudged（输出里找不到「同一把键原有的绿已撤掉 1 条」：上一版的绿按整份文件的键记进库，plan_tests 在这个样本里把 ADMISSION_BREAK 换成 verdict-lookup-skipped、不带整份文件的键，记红的键对不上那条绿）
#       features-shared-library（输出里找不到「变了 1 处：变了 fn crates/singlefs-verification/src/lib.rs::tests::shared_check）」）
#   GATE_CRASH_POINT_REPLAY_BREAK=plan-unavailable-as-failure  plan-failure 退 75 照按条复用失灵办（这一格照旧全部现判、判红），不退 75
#       unit-key-plan-unavailable、unit-key-plan-unavailable-with-red（期望退出 75，实测 1）
#   GATE_CRASH_POINT_REPLAY_BREAK=plan-unavailable-exits-at-once  plan-failure 退 75 当场整道退 75，后面的格不跑、已判红的格被吞
#       unit-key-plan-unavailable、unit-key-plan-unavailable-with-red（输出里找不到「✗ crash-case-replay 这一格没判：prune-server 不可用（上面那一句）；这一格就此收尾…」：整道当场退 75，打的是「这一道不往下跑」那一句）
#   ADMISSION_BREAK=plan-failure-not-red  归 scripts/gate/admission.py：按条复用失灵也不判红（整包那一趟退回全部现判、不判红）
#       unit-key-plan-exit-red、unit-key-plan-sentinel-red（期望退出 1，实测 3：流程判绿）
#   ADMISSION_BREAK=red-record-failure-swallowed  归 scripts/gate/admission.py：记红没成只打一行 !、照退 0，plan-failure 不撤绿
#       red-record-failed-next-pass-rejudged（输出里找不到「同一把键原有的绿已撤掉 1 条」：记红没成不撤绿）
#   GATE_CRASH_POINT_REPLAY_BREAK=red-record-failure-kept-green  记红没成只打一行「!」、不交 plan-failure（同一把键原有的绿留着，下一趟照复用）
#       red-record-failed-next-pass-rejudged（输出里找不到「同一把键原有的绿已撤掉 1 条」：记红没成只打一行「!」，不撤绿）
#   GATE_CRASH_POINT_REPLAY_BREAK=no-findings-file  不设 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE
#       green（期望退出 3，实测 1）；red（输出里找不到「第 1 节…」）；expand-up-to-narrowed-green（输出里找不到「第 1 节（stream=clean-stream states=100）…」）；
#       findings-not-written-by-the-case-green（期望退出 3，实测 1：带标记的那条不判，真没传变量的、没带标记的那条判红）；
#       judged-by-test-findings-unjudged-green、findings-judged-unmarked-red（输出里找不到「第 1 节…」）；
#       no-phase-plan-shard-merge-green、no-phase-test-plan-once-green（输出里找不到「第 1 节（stream=clean-stream states=100）…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-unfinished  没有 summary 的节当跑完了
#       red（输出里找不到「第 1 节…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-red-states  不看 red_states
#       red（输出里找不到「问题：第 1 节 red_states=3：有状态判红」）；findings-judged-unmarked-red（输出里找不到「问题：第 1 节 red_states=9：有状态判红」）；
#       judged-by-test-findings-unjudged-green（输出里找不到「crash-case:plain-stream：登记了 red-blocks=judged-by-test，发现表只打印」：发现日志判不出问题就没有这一句）
#   GATE_CRASH_POINT_REPLAY_BREAK=no-truncation  签名全打不截
#       red（输出里找不到「其余 2 个见」）
#   GATE_CRASH_POINT_REPLAY_BREAK=static-check-skipped  跳过开跑前的静态判据
#       static-check-red（输出里找不到「── 开跑前的静态判据」）
#   GATE_CRASH_POINT_REPLAY_BREAK=no-feature-pass  不起带特性那一趟
#       features-green、features-gpu-not-run、features-red、features-interrupted、features-zero、features-gpu-configured-green、features-supplement-not-run、
#       features-shared-library、features-red-pass-records-green、features-gpu-configured-judged-on-cpu、features-red-outside-the-tail、
#       features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line、cpu-judged-green-recorded
#       （带特性那一趟没起：输出里找不到它的结局句、期望退出 1 实测 3，或补判的结论报本次未跑）；
#       features-item-named-green（输出里找不到「sample-cargo-arguments: … --test store_only …」）；
#       features-domain-variable-cleared、features-gpu-last-pass（带特性那一趟没起：输出里找不到它那一趟的 sample-cargo-arguments 行）；
#       last-pass-runs-after-the-other-cells（期望退出 143，实测 3：排在最后的那一趟没起，没人发 TERM）；features-main-pass-stopped-skips-last-pass（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=features-not-from-source  特性不从源码读、写死 sample-store,gpu（假 cargo 没有这组特性的日志、退 101；假 cargo 照 cases/tests.tsv 打的样本不看特性名，不判错）
#       features-gpu-not-run、features-green、features-zero（期望退出 3，实测 1）；features-interrupted、features-red、features-red-outside-the-tail、
#       features-shared-library、cpu-judged-green-recorded、features-gpu-configured-green、features-item-named-green、
#       features-gpu-configured-cpu-judged-on-test-line、features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-judged-on-cpu、
#       features-gpu-last-pass、features-supplement-not-run（带特性那一趟找不到这组特性的日志）；
#       last-pass-runs-after-the-other-cells（期望退出 143，实测 1）；features-main-pass-stopped-skips-last-pass（输出里找不到「排在最后的目标（…）不接着跑」：带特性那一趟退 101，不算被停）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-feature-pass-exit  带特性那一趟退非 0 当成跑过
#       features-interrupted、features-red、features-red-outside-the-tail、features-red-pass-records-green、features-main-pass-stopped-skips-last-pass（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-zero-feature-files  不判挂特性的文件跑到 0 条
#       features-zero（输出里找不到「本次未跑（这不是通过）：验证包 crates/s…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=directory-target-listing-unseen  挂特性的测试目标只认 tests/ 顶层的 .rs，目录式测试目标（tests/<名>/main.rs）当不存在（feature-gated-tests 与 product-gpu-real-card-tests 两格列目标都不认）
#       features-green（输出里找不到「特性从 crates/singlefs-verification/tests 下 3 个挂特性的测试文件现读」那一句，store_only/main.rs 没列）；
#       product-gpu-green（输出里找不到「2 个对拍测试目标（特性 tests-on-a-real-gpu）每个都跑到了用例」，two_kernels_reconcile/main.rs 没列）
#   GATE_CRASH_POINT_REPLAY_BREAK=directory-target-running-line-unseen  cargo 的「Running tests/<名>/main.rs (」行认不出（源文件名里不许带 /），目录式目标跑到几条、本次未跑、ITEM_DONE 都认成 0
#       features-green（输出里找不到「✓ feature-gated-tests：3 个挂特性的测试文件（特性 gpu,sample-accelerator,sample-store）每个都编进来、跑到了用例，共 5 条」：store_only/main.rs 那一段认不出，按 0 条报本次未跑）；
#       product-gpu-green（输出里找不到「✓ product-gpu-real-card-tests：2 个对拍测试目标（特性 tests-on-a-real-gpu）每个都跑到了用例，共 3 条」：two_kernels_reconcile/main.rs 那一段认不出）
#   GATE_CRASH_POINT_REPLAY_BREAK=directory-target-name-unstripped  目录式目标的根文件 <名>/main.rs 去掉 .rs 之后不摘 /main，交给 cargo 的是 --test <名>/main
#       features-green（输出里找不到「--lib --test accelerator_items --test gpu_pair --test store_only --」：交的是 --test store_only/main，假 cargo 的特性日志照样打，别的行不变）
#   GATE_CRASH_POINT_REPLAY_BREAK=directory-target-count-by-basename  产品 GPU 包那一格数对拍目标跑到几条，按根文件的文件名（main.rs）找日志里数到的行，目录式目标（<名>/main.rs）对不上，按 0 条报本次未跑
#       product-gpu-green（输出里找不到「✓ product-gpu-real-card-tests：2 个对拍测试目标（特性 tests-on-a-real-gpu）每个都跑到了用例，共 3 条」）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-gpu-not-run  不读 NOT_RUN_NO_GPU_ADAPTER
#       features-gpu-not-run（输出里找不到「本次未跑（这不是通过）：GPU 对拍没跑…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=last-pass-merged  带 gpu 特性那一趟不把 FEATURE_PASS_LAST_TARGETS 里的测试目标拿出来另起，并在一趟里
#       features-gpu-last-pass（输出里找不到「--lib --test gpu_pair -- --nocapture」与「要跑得久的 GPU 测试目标另起一趟，排在最后」：那个目标并在一趟里）；
#       last-pass-runs-after-the-other-cells（期望退出 143，实测 3：没有另起的那一趟，没人发 TERM）；features-main-pass-stopped-skips-last-pass（输出里找不到「排在最后的目标（…）不接着跑」）
#   GATE_CRASH_POINT_REPLAY_BREAK=gpu-test-threads-one  特性里有 gpu 时测试参数带 --test-threads 1（带特性那一趟与排在最后的那一趟都带）
#       features-gpu-last-pass（输出里找不到「sample-cargo-test-threads: none」）；features-supplement-not-run（输出里找不到「sample-cargo-arguments: … --test enumeration -- --nocapture --exact --skip」）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-cell-skipped  要现判的也不起产品 GPU 包的对拍那一趟（日志空着、退出码当 0）
#       product-gpu-green、product-gpu-no-card、product-gpu-zero-tests、product-gpu-reuse-unit-change
#       （输出里找不到「sample-cargo-arguments: test --release --no-fail-fast -p singlefs-background-gpu-tests…」）；product-gpu-card-missing（输出里找不到「sample-cargo-stage: stage=record cards=[unset]」）；
#       product-gpu-red（期望退出 1，实测 3）；product-gpu-record-red（输出里找不到「✗ product-gpu-real-card-tests：真显卡对拍只录那一趟」：只录那一趟也没起）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-test-threads-one  产品 GPU 包的对拍那一趟测试参数带 --test-threads 1
#       product-gpu-reuse-unit-change（输出里找不到「sample-cargo-arguments: … --test two_kernels_reconcile -- --nocapture --exact --skip」）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-exit-ignored  产品 GPU 包的对拍那一趟 cargo test 退非 0 当成跑过
#       product-gpu-red（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-not-run-ignored  不读产品 GPU 包对拍打的 NOT_RUN_NO_GPU_ADAPTER
#       product-gpu-no-card（输出里找不到「本次未跑（这不是通过）：产品 GPU 包真显卡对拍没跑…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-zero-ignored  不判产品 GPU 包的对拍目标跑到 0 条
#       product-gpu-zero-tests（输出里找不到「本次未跑（这不是通过）：产品 GPU 包 crates/singlefs-background-gpu-tests/tests/two_kernels_reconcile.rs 挂特性…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-targets-by-file-name  对拍目标按文件名认（只认 tests/checksum 起头的），不按包与特性认
#       product-gpu-green、product-gpu-no-card、product-gpu-zero-tests、product-gpu-reuse-same-inputs、product-gpu-reuse-unrelated-change、product-gpu-reuse-unit-change、product-gpu-red、
#       product-gpu-card-missing（cfg 挂特性的 two_kernels_reconcile 认不到）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-record-holds-the-card  产品 GPU 包的测试包那一格只录那一趟也带着环境里的卡
#       product-gpu-green、product-gpu-red、product-gpu-no-card、product-gpu-zero-tests、product-gpu-reuse-unit-change、
#       product-gpu-record-red（输出里找不到「sample-cargo-stage: stage=record cards=[unset]」：只录那一趟收到了卡）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-record-exit-ignored  产品 GPU 包的测试包那一格只录那一趟退非 0 照样往下判
#       product-gpu-record-red（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-judge-without-the-stage  产品 GPU 包的测试包那一格只判那一趟不带 SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge
#       product-gpu-green、product-gpu-red、product-gpu-no-card、product-gpu-zero-tests、product-gpu-reuse-unit-change、
#       product-gpu-card-missing（输出里找不到「sample-cargo-stage: stage=judge cards=[…]」）
#   GATE_CRASH_POINT_REPLAY_BREAK=sample-caller-configuration-kept  样本档里准入模块照旧读调用方环境里 SINGLEFS_MULTI_HOST_CONFIG 指的多机配置（read_crash_case_command 不 -u 它）
#       crash-audit-caller-gpu-configuration-green（输出里找不到「sample-cargo-arguments: test --release -p singlefs-verification --test plain_stream --」：用例那一趟多带了 --features gpu）
#   GATE_CRASH_POINT_REPLAY_BREAK=service-card-not-forwarded  环境里交的卡不交给这一道自己起的那两趟 cargo（env 里 -u GPU_CARDS_BY_PRIORITY）
#       features-green、features-red、features-interrupted、features-supplement-not-run、product-gpu-green、product-gpu-red、product-gpu-no-card、product-gpu-zero-tests、
#       product-gpu-reuse-unit-change（输出里找不到「sample-cargo-gpu: cards=[192.0.2.1:0:8G] quiesced-by-the-stage=[]」：假 cargo 收到的是 cards=[unset]）
#   GATE_CRASH_POINT_REPLAY_BREAK=stage-quiesces-by-itself  环境里交了卡的那一趟自己套 scripts/crash/gpu-quiesce-run.sh（清场归 GPU 批处理验证服务，这一层本不该有；清场配置是草稿目录里只写假清场命令的一份）
#       features-green、features-red、features-interrupted、features-supplement-not-run、product-gpu-green、product-gpu-red、product-gpu-no-card、product-gpu-zero-tests、product-gpu-reuse-unit-change、
#       features-gpu-configured-cpu-judged-on-test-line、features-gpu-configured-cpu-judged-records-the-rest、
#       features-gpu-configured-green、features-gpu-configured-judged-on-cpu、features-gpu-not-run、features-gpu-last-pass、features-item-named-green、features-red-outside-the-tail、features-zero、
#       （输出里找不到「sample-cargo-gpu: cards=[…] quiesced-by-the-stage=[]」：假 cargo 收到的是 quiesced-by-the-stage=[yes]）；
#       features-main-pass-stopped-skips-last-pass、last-pass-runs-after-the-other-cells（要上卡的一趟外面多了一层清场包装）
#   GATE_CRASH_POINT_REPLAY_BREAK=red-tail-only  按条复用两趟判红时只打 cargo 输出的尾巴 40 行与「^test … FAILED$」那一条 grep：不抄红的测试目标里 FAILED 的测试函数名与 panic 段、不留整份输出
#       product-gpu-red（输出里找不到「· product-gpu-real-card-tests：红的测试目标 1 个、FAILED 的测试函数 1 条」）；
#       features-red-outside-the-tail（输出里找不到「· feature-gated-tests：红的测试目标 1 个、FAILED 的测试函数 1 条…」）；
#       red-outside-the-tail（输出里找不到「· crash-case-replay：红的测试目标 1 个、FAILED 的测试函数 1 条…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=pass-outcome-at-cell-end  每一趟 cargo 的结局不当场打：先记进草稿目录，整格收尾才打（文件头「每一趟当场写结局」；后一趟被杀时整格收不了尾，前一趟的结局跟着草稿目录没了）
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
#       progress-lines-and-bulk-lines（输出里找不到「本次未跑（这不是通过）：验证包不带特性那一趟…」）；
#       features-supplement-not-run（输出里找不到「· feature-gated-tests：不带特性那一趟打了 NOT_RUN_ 的测试函数由这一趟补判…」：收不到就没东西可补判）
#   GATE_CRASH_POINT_REPLAY_BREAK=list-items-refusals-counted-only  --list-items --with-keys 只报拒算条数、不逐条列拒算的测试函数与理由
#       样本档喂不进 --list-items（stage-selftest.sh 只传根），证红的输入是一份照 test-plan 标准输出的写法造的文件（几条 refused、几条 judge 与 reuse、末行 PLAN）：
#       GATE_CRASH_POINT_REPLAY_LIST_PLAN_OUTPUT=<那份文件> bash .claude/gate.d/crash-point-replay.sh --list-items --with-keys <根>，
#       开关关着逐条打那几行「crash-case-replay<TAB>拒算<TAB>…」，打开时一行都没有
#   GATE_CRASH_POINT_REPLAY_BREAK=judged-by-test-ignores-exit  登记了 red-blocks=judged-by-test 的用例 cargo test 退非 0 也当绿
#       judged-by-test-red-in-the-merge（期望退出 1，实测 3：核账那一趟 cargo test 退 101，带标记的用例照当绿）
#   GATE_CRASH_POINT_REPLAY_BREAK=judged-by-test-unrecognized  不认 red-blocks=judged-by-test，带标记的用例照没标记办（块那一行有红判红，不带 --ran）
#       judged-by-test-green、judged-by-test-with-red-blocks-green（期望退出 3，实测 1）；judged-by-test-missing-blocks-line、judged-by-test-missing-blocks-line-after-the-run（输出里找不到「crash-case-blocks --ran 退 4」）；
#       unmarked-red-blocks（输出里找不到「✗ 1 条崩溃枚举用例判红：crash-case:plain-stream（这一趟判绿 1 条」：带标记的那条也判红）；
#       judged-by-test-findings-unjudged-green（期望退出 3，实测 1：发现表照判）；
#       judged-by-test-fresh-equals-red-green、judged-by-test-fresh-over-red-red、unmarked-fresh-after-shards-red（输出里找不到「登记了 red-blocks=judged-by-test，按 cargo test 的退出码判（已退 0），块那一行只核在不在、数对不对得上，红块不判」）
#   GATE_CRASH_POINT_REPLAY_BREAK=ran-for-every-case  每条用例块那一行都带 --ran（没登记 red-blocks=judged-by-test 的用例块那一行有红也不判红）
#       unmarked-red-blocks（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=judged-by-test-fresh-unallowed  合并那一步核账时，登记了 red-blocks=judged-by-test 的用例也按 0 块许现判（记着红、每趟重判的红块也算各片判的块读不到）
#       judged-by-test-fresh-equals-red-green（期望退出 3，实测 1）；judged-by-test-fresh-over-red-red（输出里找不到「记着红、每趟重判的块许 2 块」：开关下写的是许 0 块）；
#       unmarked-fresh-after-shards-red（输出里找不到「✓ crash-case:fault-injection-given-states 判绿（」：同一趟里带标记的那一条在开关下也红）
#   ADMISSION_BREAK=ran-blocks-line-optional  归 scripts/gate/admission.py：crash-case-blocks --ran 没有那一行也退 0
#       judged-by-test-missing-blocks-line、judged-by-test-missing-blocks-line-after-the-run（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=judged-by-test-findings-judged  登记了 red-blocks=judged-by-test 的用例发现表照判（red_states 不是 0、发现日志缺节都判红）
#       judged-by-test-findings-unjudged-green（期望退出 3，实测 1）
#   GATE_CRASH_POINT_REPLAY_BREAK=findings-unjudged-for-every-case  每条用例的发现表都只打印、不判红（没登记的用例 red_states 不是 0 也判绿）
#       red（输出里找不到「✗ crash-case:two-signatures 的用例跑过了、日志也判得绿，发现日志却判红」）；findings-judged-unmarked-red（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=findings-not-written-unrecognized  不认 findings=not-written-by-the-case，带标记的用例发现日志缺节照判红
#       findings-not-written-by-the-case-green（期望退出 3，实测 1）
#   GATE_CRASH_POINT_REPLAY_BREAK=findings-not-written-for-every-case  每条用例都当登记了 findings=not-written-by-the-case（真没传变量的用例发现日志缺节也不判）
#       findings-judged-unmarked-red（输出里找不到「问题：用例日志里有 1 行 CRASH_POINT_REPLAY_FINDINGS」）
#   GATE_CRASH_POINT_REPLAY_BREAK=blocks-line-not-printed-unrecognized  不认 blocks-line=not-printed-by-the-case，带标记的用例没有块那一行照退 4
#       blocks-line-not-printed-green（期望退出 3，实测 1）
#   GATE_CRASH_POINT_REPLAY_BREAK=blocks-line-optional-for-every-case  每条用例块那一行都带 --no-line-registered（没登记的用例一行都没有也退 0）
#       blocks-line-not-printed-unmarked-red、judged-by-test-missing-blocks-line、judged-by-test-missing-blocks-line-after-the-run（期望退出 1，实测 3）；unmarked-red-blocks（输出里找不到「crash-case-blocks 退 1：」：红句多带了 --no-line-registered）
#   GATE_CRASH_POINT_REPLAY_BREAK=separate-run-unrecognized  不认 separate-run=<理由>：没有 --item 点名崩溃枚举用例的那一趟也跑登了它的用例、不报本次未跑
#       separate-run-not-named-skipped-green（输出里找不到「! 本次未跑（独立验证，separate-run=样本：CPU判器参照全域跑得久，单独交）：crash-case:other-stream」）
#   ADMISSION_BREAK=blocks-line-required-when-marked  归 scripts/gate/admission.py：crash-case-blocks 不认 --no-line-registered，没有那一行照退 4
#       blocks-line-not-printed-green（期望退出 3，实测 1）
#   ADMISSION_BREAK=exhaustive-judged-when-narrowed  归 scripts/gate/admission.py：crash-case-judge 不看缩域变量，缩过域的那一趟也判 exhaustive=true
#       narrowed-not-exhaustive-green（期望退出 3，实测 1）
#   ADMISSION_BREAK=exhaustive-unjudged-without-narrowing  归 scripts/gate/admission.py：判用例日志一律不判 exhaustive=true
#       not-narrowed-not-exhaustive-red（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=gpu-switch-in-key  按条复用的几趟每条测试函数的键另加 line:enable-gpu=<多机配置的 ENABLE_GPU>（样本档拿上一版算键时取上一版那一趟的配置）
#       unit-key-gpu-switch-not-in-key（输出里找不到「✓ 第 0/1 片的按条复用那一趟：第 0/1 片：没有归这一片要现判的测试函数，这一片没起 cargo（复用 3）」：上一版在 ENABLE_GPU=0 下记的判定这一版对不上键，单片那一步照旧现判）
#   GATE_CRASH_POINT_REPLAY_BREAK=withhold-whole-binary  整段不记：一段里有 NOT_RUN_ / 判段判在 CPU 的行、或没打散的结果行条数与汇总行对不上，那一段一条都不记
#       （转成 lib_libtest_result_lines.py 的 LIBTEST_RESULT_LINES_BREAK=marker-withholds-whole-binary,count-mismatch-withholds-whole-binary）
#       unit-key-not-run-withheld-per-test（输出里找不到「· 判绿不记库 2 条（这一趟现判、按条记库不认的；下一趟照跑）：」）；
#       features-supplement-not-run（输出里找不到「· 判绿记库：RECORD recorded=1 changed=0 refused=0」）；
#       progress-lines-and-bulk-lines（输出里找不到「· 判绿记库：RECORD recorded=1 changed=0 refused=0」）；
#       gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line（输出里找不到「判段判在 CPU 的记号 2 处：归到测试函数的…」：每处记号都认作整段打的）
#   GATE_CRASH_POINT_REPLAY_BREAK=library-per-pass  几趟各一份库（<阶段文件名>:crash-case-replay 与 <阶段文件名>:feature-gated-tests）
#       features-shared-library、cpu-judged-green-recorded、features-supplement-not-run、features-red-pass-records-green、
#       features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line（带特性那一趟查的是另一份库，复用的数对不上）；
#       features-domain-variable-cleared（输出里找不到「现判 crates/singlefs-verification/src/lib.rs::tests::shared_check（键对不上（上一次」：带特性那一趟的库里没有这一条）；
#       features-gpu-configured-green、features-gpu-not-run、features-green、features-zero（同一处：带特性那一趟查的是另一份库，复用的数对不上）
#   GATE_CRASH_POINT_REPLAY_BREAK=no-supplement  不补判：带特性那一趟不另带不带特性那一趟打了 NOT_RUN_ 的测试函数所在的测试目标
#       features-supplement-not-run（输出里找不到「· feature-gated-tests：不带特性那一趟打了 NOT_RUN_ 的测试函数由这一趟补判…」：enumeration_default_shape 没跑到、报本次未跑）
#   GATE_CRASH_POINT_REPLAY_BREAK=cpu-judging-withheld  判段判在 CPU 的一律不记：多机配置没开 GPU 也不给 recordable 带 --cpu-judging-recordable
#       cpu-judged-green-recorded（带特性那一趟又现判 tests::cpu_judged_check）
#       （库那一侧同一种弄坏是 lib_libtest_result_lines.py 的 LIBTEST_RESULT_LINES_BREAK=cpu-judging-always-withheld）
#   GATE_CRASH_POINT_REPLAY_BREAK=cpu-judging-recorded-when-gpu-configured  多机配置开了 GPU 也带 --cpu-judging-recordable（判段判在 CPU 的那一条照记）
#       gpu-configured-cpu-judged-not-recorded、gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line（输出里找不到「· 判绿不记库 N 条…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=sample-marker-ignored  样本根的 .gate-cells 不认，九格照跑（不带覆盖、ignore 两张表与 segments.rs 的样本在覆盖、ignore、0 次三格判红）
#       green、cpu-judged-green-recorded、crash-audit-case-log-green、crash-case-provenance-green、crash-cases-only-named-green、expand-up-to-narrowed-green、features-domain-variable-cleared、
#       features-gpu-configured-cpu-judged-on-test-line、features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-green、features-gpu-configured-judged-on-cpu、features-gpu-not-run、
#       features-gpu-last-pass、features-green、features-red-pass-records-green、features-shared-library、features-supplement-not-run、features-zero、gpu-configured-plain-pass-not-run、
#       judged-by-test-green、judged-by-test-with-red-blocks-green、judged-by-test-findings-unjudged-green、findings-not-written-by-the-case-green、blocks-line-not-printed-green、separate-run-not-named-skipped-green、separate-run-named-runs-green、narrowed-not-exhaustive-green、product-gpu-card-missing、product-gpu-green、product-gpu-no-card、product-gpu-no-target、
#       product-gpu-reuse-same-inputs、product-gpu-reuse-unit-change、product-gpu-reuse-unrelated-change、product-gpu-zero-tests、registered-case-skipped-in-the-plain-pass、unit-key-gpu-switch-not-in-key、
#       unit-key-not-run-withheld-per-test、unit-key-samples-green、progress-lines-and-bulk-lines、
#       case-binaries-copied-without-debug-info、judged-by-test-fresh-equals-red-green、no-phase-plan-shard-merge-green、no-phase-test-plan-once-green、
#       package-pass-shards-run-the-plan-binaries-without-building、shard-builds-itself-when-the-plan-binary-is-missing、shards-run-the-plan-binaries-without-building（期望退出 3，实测 1：点名格少于九格，出口 4 有格判红）；
#       crash-case-provenance-red、features-red、product-gpu-red（输出里找不到「✗ 有 1 格判红：…」）；
#       crash-case-no-card-for-observer-images-not-run（九格照跑，别的格判红，整道退不成 78）
#   GATE_CRASH_POINT_REPLAY_BREAK=crash-audit-red-ignored  出口 4 那五格与 crash-case-provenance 红了也不算进整道（不进红格）
#       crash-audit-cells-red（期望退出 1，实测 3）；
#       crash-audit-flow-records-unavailable（期望退出 75，实测 3：不可用的三格也被清出整道）；crash-audit-flow-records-unavailable-with-red（期望退出 1，实测 3）；crash-case-provenance-red（期望退出 1，实测 3：来历表缺行的红也被清出整道）
#   GATE_CRASH_POINT_REPLAY_BREAK=flow-records-unavailable-as-failure  collect 退 75 照判法脚本出错办（覆盖、ignore、0 次三格判红）
#       crash-audit-flow-records-unavailable（期望退出 75，实测 1）；crash-audit-flow-records-unavailable-with-red（输出里找不到「✗ crash-flow-coverage 这一格没判：prune-server 不可用（…」）
#   static-check 格与出口 4 那几格的判法开关（CRASH_CASE_CHECK_BREAK、CRASH_FLOW_COVERAGE_BREAK）不在这里列：scripts/crash/crash-case-check.py、
#   scripts/crash/crash-flow-coverage.py 平常跑时设了开关整份退 2（只在 --selftest 里认），在样本上证不了；开关表在它们文件头，由它们的 --selftest 证红
#   GATE_CRASH_POINT_REPLAY_BREAK=red-not-recorded  按条复用的几趟判红时不把 FAILED 的测试函数交 verdict-record --red-names（门禁判红的不记红、不撤同一把键原有的绿），判绿的照记
#       red-rejudged-next-pass（输出里找不到「· 判红记库：1 条记红、撤掉同一把键原有的绿，下一趟照现判（RECORD recorded=0 changed=0 refused=0…」）；
#       red-record-failed-next-pass-rejudged（输出里找不到「同一把键原有的绿已撤掉 1 条」：不交红的就没有记红没成那一句、不撤绿）；
#       red-pass-records-green、features-red-pass-records-green、product-gpu-red（输出里找不到「· 判红记库：1 条记红、撤掉同一把键原有的绿，下一趟照现判（RECORD recorded=…」）；
#       gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line（同一句：判段判在 CPU 归得到的那几条没记红）
#   GATE_CRASH_POINT_REPLAY_BREAK=plain-pass-red-withholds-green  crash-case-replay 格不带特性那一趟 cargo test 退非 0 时不认结局、判绿的一条都不记（只记红）
#       red-pass-records-green（输出里找不到「· 判红记库：1 条记红、撤掉同一把键原有的绿，下一趟照现判（RECORD recorded=1 changed=0 refused=0…」：green_check 没记绿）
#   GATE_CRASH_POINT_REPLAY_BREAK=feature-pass-red-withholds-green  feature-gated-tests 格带特性那一趟 cargo test 退非 0 时不认结局、判绿的一条都不记（只记红）
#       features-red-pass-records-green（输出里找不到「· 判红记库：1 条记红、撤掉同一把键原有的绿，下一趟照现判（RECORD recorded=1 changed=0 refused=0…」：store_only_quick 没记绿）
#   GATE_CRASH_POINT_REPLAY_BREAK=product-gpu-red-withholds-green  product-gpu-real-card-tests 格对拍那一趟 cargo test 退非 0 时不认结局、判绿的一条都不记（只记红）
#       product-gpu-red（输出里找不到「· 判红记库：1 条记红、撤掉同一把键原有的绿，下一趟照现判（RECORD recorded=2 changed=0 refused=0…」：判绿的另 2 条没记）
#   GATE_CRASH_POINT_REPLAY_BREAK=test-functions-request-gate-environment  test-functions-request 不带 cargo 前面那一串 env -u，按门禁自己没清的环境起（键按没生效的变量算）
#       features-domain-variable-cleared（输出里找不到「新的输入行 variable SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO unset」：带特性那一趟的键还带着变量的 3，shared_check 照键复用，现判只有 1 条）
#   GATE_CRASH_POINT_REPLAY_BREAK=plain-pass-cpu-judging-records-nothing  crash-case-replay 格不带特性那一趟配了 GPU、日志里有判段判在 CPU 的行时一条都不记（不调 record_cpu_judging_pass）
#       gpu-configured-cpu-judged-records-the-rest（输出里找不到「· crash-case-replay：判段判在 CPU 的记号 2 处：…」：cpu_judged_check 没记红、settled_check 与 other_green 没记绿）；
#       gpu-configured-cpu-judged-not-recorded（输出里找不到「· 判绿不记库 1 条（这一趟现判、按条记库不认的；下一趟照跑）：」：不调 record_cpu_judging_pass，没有记库那一步打的句子）
#   GATE_CRASH_POINT_REPLAY_BREAK=feature-pass-cpu-judging-records-nothing  feature-gated-tests 格带 gpu 特性那一趟配了 GPU、日志里有判段判在 CPU 的行时一条都不记（不调 record_cpu_judging_pass）
#       features-gpu-configured-cpu-judged-records-the-rest、features-gpu-configured-cpu-judged-on-test-line（输出里找不到「· feature-gated-tests：判段判在 CPU 的记号 2 处：…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=cpu-judging-line-start-only  cpu_judging_lines 只认去掉行首空白后以记号起头的行（接在「test <名> ... 」同一行后面的认不出）
#       features-gpu-configured-cpu-judged-on-test-line（期望退出 1，实测 3：两处记号都接在 test 名后面，带 gpu 特性那一趟一处判在 CPU 都没认出）
#   逐项点名（文件头「逐项点名」）的四个：
#   GATE_CRASH_POINT_REPLAY_BREAK=feature-items-ignored  点了 feature-gated 的项也照整格跑（--lib 与全部挂特性的文件）
#       features-item-named-green（输出里找不到「· feature-gated-tests：只编、只跑 --item 点名的 1 项：feature-gated:store_only.rs」）
#   GATE_CRASH_POINT_REPLAY_BREAK=feature-item-features-narrowed  点了 feature-gated 的项时特性只取点名的文件的（键与整格的不同，点名单跑与整格判的不是同一份）
#       features-item-named-green（期望退出 3，实测 1：特性只剩 sample-store，假 cargo 没有这组特性的日志）
#   GATE_CRASH_POINT_REPLAY_BREAK=feature-item-unknown-accepted  点了 tests/ 里没有的文件也照收（悄悄丢掉，不退 2）
#       features-item-unknown（期望退出 2，实测 3：丢掉之后一项都没点，照整格跑）
#   GATE_CRASH_POINT_REPLAY_BREAK=item-done-unreported  跑完不打 ITEM_DONE（feature-gated 的项与崩溃枚举用例都不打）
#       features-green、features-item-named-green（输出里找不到「ITEM_DONE item=feature-gated:…」）；
#       crash-cases-only-named-green（输出里找不到「ITEM_DONE item=crash-case:plain-stream」）；
#       crash-case-no-card-for-observer-images-not-run（输出里找不到「judged=0 reused=0 exit=78」）
#   GATE_CRASH_POINT_REPLAY_BREAK=no-card-for-observer-images-as-red  不认观察者交回的盘取不到卡那一行：那一趟照 cargo 退非 0 判红
#       crash-case-no-card-for-observer-images-not-run（期望退出 78，实测 1）
#   --crash-cases-only（文件头「--crash-cases-only」）的两个：
#   GATE_CRASH_POINT_REPLAY_BREAK=crash-cases-only-ignored  带了 --crash-cases-only 也照起按条复用那一趟（参数照收、不起作用）
#       crash-cases-only-named-green（期望退出 3，实测 1：按条复用那一趟起了，三条要现判的测试函数照 cases/default-pass 判红）；
#       case-binaries-copied-without-debug-info、shard-builds-itself-when-the-plan-binary-is-missing、shard-plan-shards-at-least-sixteen、shard-plan-small-named-case-one-shard、
#       shards-run-the-plan-binaries-without-building（按条复用那一趟照起：分片的计划句子多一段「包里按条复用那一趟要现判的测试目标」，计划那一步多算一遍 test-plan、多编一回按条复用那一趟的测试二进制，cargo 起的遍数与计划、各片的句子对不上）
#   GATE_CRASH_POINT_REPLAY_BREAK=crash-cases-only-items-unchecked  带了 --crash-cases-only 而没点崩溃枚举用例也照收（不退 2，登记表里的用例全走按块那一路）
#       crash-cases-only-without-item（期望退出 2，实测 1：没点的 other-stream 也起了，假 cargo 没有它的日志、照 cases/default-pass 判红）
#   进度行与逐提交的行（文件头「进度行与逐提交的行」）的五个：
#   GATE_CRASH_POINT_REPLAY_BREAK=only-replay-progress-forwarded  进度行只转 CRASH_POINT_REPLAY_PROGRESS 一种，别的前缀全丢
#       progress-lines-and-bulk-lines（输出里找不到「CRASH_JUDGE_GPU_PAIRING_PROGRESS flow=sample」）
#   GATE_CRASH_POINT_REPLAY_BREAK=pass-activity-silent  cargo 有输出而没有进度行时也不报那一行
#       progress-lines-and-bulk-lines（输出里找不到「CRASH_POINT_REPLAY_PASS_ACTIVE seconds=」）
#   GATE_CRASH_POINT_REPLAY_BREAK=bulk-lines-kept  逐提交的行照进日志，不滤
#       progress-lines-and-bulk-lines（输出里找不到「· crash-case-replay：不带特性那一趟的逐提交的行没进日志：…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=bulk-lines-with-marks-dropped  夹着判法要认的记号的那一行也滤掉（判法读的行少了）
#       progress-lines-and-bulk-lines（输出里找不到「· crash-case-replay：不带特性那一趟的逐提交的行没进日志：GPU_LIFECYCLE 4 行…」：夹着 NOT_RUN_ 的那一行也算进了滤掉的、记号没了）
#   GATE_CRASH_POINT_REPLAY_BREAK=bulk-tail-dropped  逐提交的行不留末尾现场
#       progress-lines-and-bulk-lines（输出里找不到「…；末尾现场 3 行在 」）
#   红了接着跑，停了不接着跑（文件头同名一段）的两个：
#   GATE_CRASH_POINT_REPLAY_BREAK=package-pass-red-skips-cases  不带特性那一趟红了不跑登记的崩溃枚举用例
#       没有样本打得到：没有分片阶段时不带特性那一趟在单片那一步红了、整道在那里退 1，这一句只在合并那一步按条复用算不出时才走到（服务交 TASK_SERVER_SHARD_PHASE=merge 才起得到）
#   GATE_CRASH_POINT_REPLAY_BREAK=stopped-pass-runs-the-rest  不认这一趟被停了：后面的用例、格与排在最后的那一趟接着跑
#       plain-pass-stopped-skips-the-rest、plain-pass-test-binary-terminated-skips-the-rest（输出里找不到「本次未跑（这不是通过）：crash-case-replay 第 0 片：这一趟…被停了…不接着跑」）；
#       crash-case-stopped-skips-the-rest（输出里找不到「…后面 1 条崩溃枚举用例不接着跑：crash-case:other-stream」）；
#       features-main-pass-stopped-skips-last-pass（输出里找不到「排在最后的目标（…）不接着跑」）
#   格的次序（文件头同名一段）的两个：
#   GATE_CRASH_POINT_REPLAY_BREAK=last-pass-before-the-other-cells  带特性那一格（连同排在最后的那一趟）排在产品 GPU 与只读树的格之前
#       last-pass-runs-after-the-other-cells（输出里找不到「· product-gpu-real-card-tests：…无对象可判」：排在最后的那一趟的假 cargo 先把阶段停了）
#   GATE_CRASH_POINT_REPLAY_BREAK=feature-main-pass-recorded-with-last-pass  带特性那一趟的判定不先记，等排在最后的那一趟跑完
#       last-pass-runs-after-the-other-cells（输出里找不到「· feature-gated-tests：要跑得久的目标排在最后，带特性那一趟的判定先记：」）
#   GATE_CRASH_POINT_REPLAY_BREAK=pass-output-program-in-scratch  接输出的那一段的程序写进草稿目录的一份文件再起；样本里有假 cargo 停阶段（cases/*.term-parent）的，这一段等到草稿目录被删才起（至多 20 秒；假 cargo 等不到接输出的那一段接上手，等满 10 秒才发 TERM），打不开程序
#       killed-pass-keeps-earlier-outcome（输出里找不到「CRASH_POINT_REPLAY_PROGRESS sample-cargo-arguments: test --release -p singlefs-verification --features sample-store --test fault_injection_given_states」：被停那一趟已打的行丢了）
#   GATE_CRASH_POINT_REPLAY_BREAK=sample-service-unchecked  进样本档不核在不在临时 prune-server 底下（裸跑也判，样本的判定写进真服务）
#       sample-root-bare-red（期望退出 2，实测照判 crash-case-replay 那一格；输出里找不到「没有临时 prune-server」）
crash_point_replay_range_given=0
root_argument=""
crash_point_replay_list_cells=0
crash_point_replay_list_items=0
crash_point_replay_list_with_keys=0
crash_point_replay_named_cases=()
crash_point_replay_crash_cases_only=0   # --crash-cases-only（文件头「--crash-cases-only」）
# 格名表：与文件头九行 `# gate-cell:` 逐字相同、次序相同
CRASH_CASE_REPLAY_CELL_NAME="crash-case-replay"
CRASH_CASE_REPLAY_CELL_TITLE="包里全部测试函数按单元键复用（登记的崩溃枚举用例经 --skip 排掉）、登记的崩溃枚举用例逐条按崩溃状态块的单元键跑（块键相同的复用、缺的当场判；登记行第三列带 separate-run= 的除外，没被 --item 点名时逐条报本次未跑）；带 --crash-cases-only 时只走 --item 点名的崩溃枚举用例按块那一路、不起按条复用那一趟"
FEATURE_GATED_TESTS_CELL_NAME="feature-gated-tests"
# 带 gpu 特性那一趟里另起一趟、排在最后的测试目标（文件头「格的次序」：要跑的时间比别的加起来长得多，别的目标的判定先记下）
FEATURE_PASS_LAST_TARGETS=(crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state)
FEATURE_GATED_TESTS_CELL_TITLE="带上 验证包 tests/ 里 cfg 点到的特性另跑一趟（每条测试函数按单元键复用）：挂特性的测试文件逐个跑到或复用的用例不为 0，GPU 对拍没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑"
PRODUCT_GPU_REAL_CARD_CELL_NAME="product-gpu-real-card-tests"
PRODUCT_GPU_REAL_CARD_CELL_TITLE="起产品 GPU 包的测试包（singlefs-background-gpu-tests）带 tests-on-a-real-gpu 特性才编的真显卡对拍测试（每条测试函数按单元键复用）：对拍判红整格红，没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑"
# 产品的后台 GPU 包与它的真显卡对拍特性（Cargo.toml 的 [features]；测试目标靠 [[test]] 的 required-features 或测试文件的 cfg 挂在它后面，product_gpu_target_rows 现读）
PRODUCT_GPU_PACKAGE_NAME="singlefs-background-gpu-tests"
PRODUCT_GPU_FEATURE_NAME="tests-on-a-real-gpu"
# --check 点名的格（空着就是九格都跑）
crash_point_replay_selected_cells=()
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
if ! declare -F preflight >/dev/null; then
  echo "  ✗ crash-point-replay.sh：source .claude/scripts/preflight.sh 没成（上面一行报了找不到），preflight 没有定义、参数已被清空，拒绝往下跑（不拦的话 --check 与 --list 会变成全跑）"
  echo "     → 怎么办：这棵树缺 .claude/scripts/preflight.sh：建副本时连 .claude/scripts 与 .claude/singlefs-ai-sop 一起拷（scripts/hosts/minimal-copy.py 逐条加 --with .claude/scripts=… 与 --with .claude/singlefs-ai-sop=…），或回主工作区跑"
  exit 2
fi
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
# 只读树的三格：不等带特性那一趟，排在它前面（文件头「格的次序」）
CRASH_AUDIT_READ_ONLY_CELLS=(crash-case-must-red crash-knobs crash-case-provenance)
# 合表各项的来历表核对（不属出口 4 那五格，判法同样在 crash-case-check.py，只读树）
CRASH_CASE_PROVENANCE_CELL_NAME="crash-case-provenance"
CRASH_CASE_PROVENANCE_CELL_TITLE="崩溃点重放合表里调 enumerate_crash_states 的崩溃枚举项与放量分项，在来历表 crash_point_replay_provenance.rs 里都有一行"
# 函数分在 lib/ 下的四份库里（说明见各份库头），在格名表与 preflight 之后、参数解析之前按下面的次序 source；它们读的变量在调用时才取
source "$(dirname "${BASH_SOURCE[0]}")/lib/crash-point-replay-selection-and-plan.sh"
source "$(dirname "${BASH_SOURCE[0]}")/lib/crash-point-replay-package-pass-verdicts.sh"
source "$(dirname "${BASH_SOURCE[0]}")/lib/crash-point-replay-crash-case-cells.sh"
source "$(dirname "${BASH_SOURCE[0]}")/lib/crash-point-replay-feature-card-and-audit-cells.sh"
# crash_point_replay_test_functions_request <阶段文件名> <判法根>：按条复用的请求（判定种类、交给 test-plan 的参数）只在 scripts/gate/admission.py 的
#   test-functions-request 一处（它文件头「按条复用的共用段」；scripts/crash/crash-point-replay.sh 按项跑的那一段调同一条，两边记进、读的是同一类判定库），
#   放进全局 crash_point_replay_test_functions_kind 与数组 crash_point_replay_test_functions_arguments。读不出时种类写 <阶段文件名>:test-functions-request-unreadable、
#   参数写成一个 test-plan 认不出的 --token（那一趟 test-plan 退 2，照文件头「复用」里「按条复用失灵」判红，不悄悄换一把键）。
#   test-functions-request 在交给 cargo 的同一份环境里起（admission.py 的 DOMAIN_ENVIRONMENT_VARIABLES 按那一刻的环境把改判定域的变量写进键）：带特性那一趟与产品 GPU 真显卡那一趟
#   cargo 前面是 env -u 清掉调用方 SINGLEFS_CRASH_AMPLIFICATION_* 的那一串（cleared_environment），这两格先把那一串放进全局数组 crash_point_replay_test_functions_environment，
#   这里用 env 带着它起 test-functions-request；不带特性那一趟 cargo 不清，数组是空的。弄坏开关 test-functions-request-gate-environment 下不带（按门禁自己没清的环境起）
crash_point_replay_test_functions_environment=()
# 逐项点名（文件头「逐项点名」）：feature-gated 的项的前缀、库那一项的名字
FEATURE_GATED_ITEM_PREFIX="feature-gated:"
FEATURE_GATED_LIBRARY_ITEM="feature-gated:lib"
crash_point_replay_named_feature_items=()   # --item 点名的 feature-gated 项（带前缀）
while (($#)); do
  stage_argument="$1"
  case "$stage_argument" in
    --list) crash_point_replay_list_cells=1 ;;
    --list-items) crash_point_replay_list_items=1 ;;
    --with-keys) crash_point_replay_list_with_keys=1 ;;
    --crash-cases-only) crash_point_replay_crash_cases_only=1 ;;
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
      echo "     → 怎么办：只认 --list、--check $CRASH_CASE_REPLAY_CELL_NAME、--list-items [--with-keys]、--item <项>、--crash-cases-only 与项目根，顺序不限：bash .claude/gate.d/crash-point-replay.sh [--check <格名>[,<格名>…]] [--item <项>] [--crash-cases-only] [项目根]"
      exit 2 ;;
    *) root_argument="$stage_argument" ;;
  esac
  shift
done
# 分片（文件头「分片」）：服务给的这一步、片号与两个目录读完就从环境里清掉，不传给 cargo、准入模块与用例
crash_point_replay_shard_phase="${TASK_SERVER_SHARD_PHASE:-}"
crash_point_replay_shard_position="${TASK_SERVER_SHARD:-}"
crash_point_replay_shard_plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}"
crash_point_replay_shard_missing="${TASK_SERVER_SHARD_MISSING:-}"
crash_point_replay_shard_output_directory="${TASK_SERVER_SHARD_OUTPUT_DIRECTORY:-}"
unset TASK_SERVER_SHARD_PHASE TASK_SERVER_SHARD TASK_SERVER_SHARD_PLAN_DIRECTORY TASK_SERVER_SHARD_OUTPUT_DIRECTORY TASK_SERVER_SHARD_OUTPUTS \
  TASK_SERVER_SHARD_MISSING TASK_SERVER_SHARD_HOST
crash_point_replay_shard_index=0
crash_point_replay_shard_count=1
# 计划那一步算好的不带特性那一趟的 test-plan 放的目录（文件头「分片」plan）：服务交的计划目录下的 test-plan/；没有分片阶段时自己走的计划那一步放草稿目录里（下面主体里设）
crash_point_replay_shared_plan_directory=""
[[ -z "$crash_point_replay_shard_plan_directory" ]] || crash_point_replay_shared_plan_directory="$crash_point_replay_shard_plan_directory/test-plan"
# 计划那一步放成了 test-plan（crash_point_replay_store_shared_plan 置 1）；这一片读的是计划那一步的 test-plan（crash_point_replay_plan_for_package_pass 置 1）；
#   这一片按条复用那一趟交给 test-targets-parallel.py --binaries 的清单（crash_point_replay_settle_package_pass_binaries 写，空串是不交）
crash_point_replay_shared_plan_stored=0
crash_point_replay_shared_plan_read=0
crash_point_replay_package_binaries=""
# 这一片不起的测试目标：别的片领的目标根文件（计划里 judge、refused 行第五列的写法，一行一个），crash_point_replay_keep_shard_rows 写；空串是没有（不分片、计划算不出）
crash_point_replay_other_shard_roots_file=""
# 计划那一步预先编好的登记的崩溃枚举用例的测试二进制放的目录（文件头「分片」plan）：服务交的计划目录下的 test-binaries/，各片与合并读它；
#   没有分片阶段时这一件不预先编（样本根的 .gate-shard-count 点名几片的照服务那样预先编，放草稿目录里，下面主体里设）
crash_point_replay_shared_binaries_directory=""
[[ -z "$crash_point_replay_shard_plan_directory" ]] || crash_point_replay_shared_binaries_directory="$crash_point_replay_shard_plan_directory/test-binaries"
# 服务交的这一步（没有分片阶段是空串）：没有分片阶段时这一件里 crash_point_replay_shard_phase 会依次改成 shard、merge，
#   谁删计划那一步放的测试二进制（crash_point_replay_drop_shared_binaries）只看这一个；片判绿收尾、计划写成了 plan.json 各置 1
crash_point_replay_service_shard_phase="$crash_point_replay_shard_phase"
crash_point_replay_shard_step_green=0
crash_point_replay_shard_plan_written=0
case "$crash_point_replay_shard_phase" in
  ""|plan|merge) ;;
  shard)
    if [[ ! "$crash_point_replay_shard_position" =~ ^([0-9]+)/([1-9][0-9]*)$ ]] || (( BASH_REMATCH[1] >= BASH_REMATCH[2] )) \
       || [[ "$(head -n 1 -- "$crash_point_replay_shard_plan_directory/shards" 2>/dev/null)" != "${BASH_REMATCH[2]}" ]]; then
      crash_point_replay_shard_refuse "片号 TASK_SERVER_SHARD=${crash_point_replay_shard_position:-（没给）} 对不上计划目录 ${crash_point_replay_shard_plan_directory:-（没给）} 里的片数" \
        "这一步只由 task-server 照计划那一步写的 plan.json 起" 2
    fi
    crash_point_replay_shard_index="${BASH_REMATCH[1]}"
    crash_point_replay_shard_count="${BASH_REMATCH[2]}"
    # 片退 0 就要有产物目录（task-server/task_server_lib.py 文件头「分片」②）；这一道的结论不经它交（判定在服务里），只建、空着
    [[ -n "$crash_point_replay_shard_output_directory" ]] && mkdir -p -- "$crash_point_replay_shard_output_directory" \
      || crash_point_replay_shard_refuse "建不了产物目录 ${crash_point_replay_shard_output_directory:-（没给）}" "看共享工作根可不可写" 78 ;;
  *) crash_point_replay_shard_refuse "TASK_SERVER_SHARD_PHASE=$crash_point_replay_shard_phase 认不得（只认 plan、shard、merge）" "照 task-server/task_server_lib.py 文件头「分片」的协议起" 2 ;;
esac
if [[ "$crash_point_replay_shard_phase" == merge && -n "$crash_point_replay_shard_missing" ]]; then
  echo "  ! 本次未跑（这不是通过）：分片的合并那一步缺片 $crash_point_replay_shard_missing（人手取消了那几片），一格都不判"
  echo "     → 怎么办：重交整件（同一条命令），各片判过的照单元键复用"
  # 合并那一步收尾一律删计划那一步放的测试二进制（文件头「分片」：谁删）；这里在 EXIT trap 之前退，自己删
  [[ -z "$crash_point_replay_shared_binaries_directory" ]] || rm -rf -- "$crash_point_replay_shared_binaries_directory"
  exit 77
fi
if ((crash_point_replay_list_cells)); then
  [[ "$crash_point_replay_shard_phase" == plan ]] && crash_point_replay_write_shard_plan 0 "--list 只打格名表，不分片"
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
      # 拒算的逐条列：测试函数与理由（test-plan 一行的第一列与第七列，理由去掉「拒算：」起头），一条不落；弄坏开关 list-items-refusals-counted-only 下不列
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
  # 调用方交的卡同样只从 caller-environment 来（文件头「显卡」）：调用方环境里的那一份清掉，判错的样本不随经不经 task-server、交没交卡变
  unset GPU_CARDS_BY_PRIORITY
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
# cargo 的「Running <源文件> (」行里测试目标源文件的名字怎么取：名字里可以带 /（平铺的 tests/<名>.rs、目录式的 tests/<名>/main.rs 都认，五处 awk 共用）；
#   弄坏开关 directory-target-running-line-unseen 下名字里不许带 /，目录式目标那一段认不出
crash_point_replay_running_name='[^ ]'; [[ "$crash_point_replay_break" != directory-target-running-line-unseen ]] || crash_point_replay_running_name='[^/ ]'
# 目录式测试目标的根文件 <名>/main.rs 去掉 .rs 之后的 /main 要摘掉才是 cargo 的 --test 参数（<名>）；弄坏开关 directory-target-name-unstripped 下不摘，交出 --test <名>/main
crash_point_replay_directory_main='/main'; [[ "$crash_point_replay_break" != directory-target-name-unstripped ]] || crash_point_replay_directory_main=''
((crash_point_replay_sample_mode == 0)) || crash_point_replay_sample_service_check
crash_point_replay_apply_sample_marker
crash_point_replay_apply_sample_items
crash_point_replay_apply_sample_crash_cases_only
crash_point_replay_apply_sample_shard_phase
# --crash-cases-only 要点名崩溃枚举用例（文件头「--crash-cases-only」）：一条都没点退 2。弄坏开关 crash-cases-only-items-unchecked 下照收
if (( crash_point_replay_crash_cases_only && ${#crash_point_replay_named_cases[@]} == 0 )) && [[ "$crash_point_replay_break" != crash-cases-only-items-unchecked ]]; then
  echo "  ✗ 带了 --crash-cases-only 却没点崩溃枚举用例：它只走按块那一路、只跑 --item 点名的用例，一条都没点就没有要跑的"
  echo "     → 怎么办：加 --item crash-case:<用例名>（几条给几次；可点的用 bash .claude/gate.d/crash-point-replay.sh --list-items 看）。全部登记的用例连同包里按条复用那一趟是不带它的整格，归提交时的门禁"
  exit 2
fi
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

# 分片的计划那一步（文件头「分片」plan）：参数、静态判据与前提上面都判过了；没点 crash-case-replay 格写 0 片、退 0。点了的在这里先定片数的上界：
#   多机配置点名的卡数与点名用例登记的 shards-at-least= 的大者（开了 GPU 取 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED，没写取 GPU_CARDS_BY_PRIORITY；没开 GPU 取 HOSTS 的台数），
#   放进 crash_point_replay_shard_plan_count、crash_point_replay_shard_plan_origin；片数本身等下面主体里算完 test-plan、预先编完测试二进制，照这一趟要现判的量定（crash_point_replay_decide_shard_count）
if [[ "$crash_point_replay_shard_phase" == plan ]]; then
  crash_point_replay_runs_cell "$CRASH_CASE_REPLAY_CELL_NAME" \
    || crash_point_replay_write_shard_plan 0 "这一趟没点 $CRASH_CASE_REPLAY_CELL_NAME 格，点了的格在合并那一步跑"
  [[ -n "$crash_point_replay_configuration_file" && -f "$crash_point_replay_configuration_file" ]] \
    || crash_point_replay_shard_refuse "找不到多机配置（${crash_point_replay_configuration_file:-crash-amplification-lib.sh 的找法没找到}）：定不了分几片" \
      "照仓根 multi-host.env.example 写好多机配置（SINGLEFS_MULTI_HOST_CONFIG 或主工作树根下的 multi-host.env）再交" 78
  crash_point_replay_shard_source="HOSTS"
  if (( crash_point_replay_gpu_configured )); then crash_point_replay_shard_source="GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED"; fi
  crash_point_replay_shard_value="$(source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_configuration_value "$crash_point_replay_configuration_file" "$crash_point_replay_shard_source")"
  if [[ -z "$crash_point_replay_shard_value" ]] && (( crash_point_replay_gpu_configured )); then
    crash_point_replay_shard_source="GPU_CARDS_BY_PRIORITY"
    crash_point_replay_shard_value="$(source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_configuration_value "$crash_point_replay_configuration_file" "$crash_point_replay_shard_source")"
  fi
  crash_point_replay_shard_words=()
  read -r -a crash_point_replay_shard_words <<<"$crash_point_replay_shard_value"
  (( ${#crash_point_replay_shard_words[@]} > 0 )) \
    || crash_point_replay_shard_refuse "多机配置 $crash_point_replay_configuration_file 的 $crash_point_replay_shard_source 是空的：定不了分几片" \
      "开了 GPU 的写上点名的卡（GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 或 GPU_CARDS_BY_PRIORITY），没开的写上 HOSTS，再交" 78
  # 点名的用例里登记了 shards-at-least= 的（状态多、照卡数分的片一片跑不完，.claude/rules/long-run-split.md）：片数取它与卡数的大者
  crash_point_replay_shard_plan_count="${#crash_point_replay_shard_words[@]}"
  crash_point_replay_shard_plan_origin="多机配置 $crash_point_replay_shard_source 点名 ${#crash_point_replay_shard_words[@]} 项，一项一片"
  if (( ${#crash_point_replay_named_cases[@]} > 0 )); then
    if crash_point_replay_shards_at_least="$(python3 "$crash_point_replay_admission_module" crash-case-shards-at-least "$ROOT" "${crash_point_replay_named_cases[@]}")"; then
      [[ "$crash_point_replay_shards_at_least" =~ ^[0-9]+$ ]] \
        || crash_point_replay_shard_refuse "admission.py crash-case-shards-at-least 打的不是一个数：$crash_point_replay_shards_at_least" \
          "交 task-server/task submit -- python3 scripts/gate/admission.py --selftest 看这个子命令" 2
      if (( crash_point_replay_shards_at_least > crash_point_replay_shard_plan_count )); then
        crash_point_replay_shard_plan_count="$crash_point_replay_shards_at_least"
        crash_point_replay_shard_plan_origin="点名的用例登记了 shards-at-least=$crash_point_replay_shards_at_least，多于多机配置 $crash_point_replay_shard_source 点名的 ${#crash_point_replay_shard_words[@]} 项，照登记的分"
      fi
    else
      crash_point_replay_shard_refuse "点名的崩溃枚举用例至少分几片答不出：$crash_point_replay_shards_at_least" \
        "照 .claude/gate.d/stage-inputs.tsv 里那几条 crash-case: 行核第三列（shards-at-least=<正整数>，至多一条），改了再交" 2
    fi
  fi
fi

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
trap crash_point_replay_exit_trap EXIT
if ! crash_point_replay_pass_log_directory="$(mktemp -d "${TMPDIR:-/tmp}/crash-point-replay-passes.XXXXXX")"; then
  crash_point_replay_pass_log_directory=""
  echo "  ✗ 建不了放每一趟 cargo 整份输出的目录（mktemp -d 在 ${TMPDIR:-/tmp} 下失败），一趟 cargo 都没起"
  echo "     → 怎么办：看 ${TMPDIR:-/tmp} 在不在、可不可写、盘满没满（df -h ${TMPDIR:-/tmp}），修好之后重跑这一道"
  exit 1
fi
# 每一趟 cargo 输出的那一段管道（文件头「进度行与逐提交的行」）：末尾现场每一代的行数、没进度行时多久报一行还有输出；样本档照标记目录里的两份文件改小，不等真的这么久、不造这么多行
crash_point_replay_bulk_tail_lines=20000
crash_point_replay_activity_seconds=300
if [[ "$crash_point_replay_sample_mode" == 1 ]]; then
  [[ -f "$PWD/.crash-point-replay-sample-tools/bulk-tail-lines" ]] && crash_point_replay_bulk_tail_lines="$(< "$PWD/.crash-point-replay-sample-tools/bulk-tail-lines")"
  [[ -f "$PWD/.crash-point-replay-sample-tools/activity-seconds" ]] && crash_point_replay_activity_seconds="$(< "$PWD/.crash-point-replay-sample-tools/activity-seconds")"
fi
crash_point_replay_pass_output_program="$(cat <<'PY_PASS_OUTPUT'
# 文件头「进度行与逐提交的行」：cargo 的输出从标准输入进来，别的行原样进日志，进度行另转到标准输出，逐提交的行只留计数与末尾现场
import os
import re
import select
import signal
import sys
import time

log_path, break_switch, tail_generation_lines, activity_seconds = sys.argv[1], sys.argv[2], int(sys.argv[3]), float(sys.argv[4])
BULK_PREFIXES = (b"GPU_LIFECYCLE ", b"CRASH_JUDGE_ACCOUNTING_SLICE ")
# 判定在一行中间认的记号（lib_libtest_result_lines.py 的 NOT_RUN_、判段判在 CPU，crash-flow-coverage.py 的覆盖行与 I-x.y= 计数）与打散在这一行里的结果行、panic：夹着这些的行照进日志
KEPT_MARKS = re.compile(rb"NOT_RUN_|CRASH_|_PROGRESS|I-[0-9]|test result: | \.\.\. |panicked at")
FORWARDED_LINE = re.compile(rb"^CRASH_POINT_REPLAY_PROGRESS\s" if break_switch == "only-replay-progress-forwarded"
                            else rb"(?:^|\s)(?:[A-Z][A-Z0-9_]*_PROGRESS|CRASH_AMPLIFICATION_BLOCK)\s")
HELD_LINE_BYTES = 1 << 20
TAIL_PATH, PREVIOUS_TAIL_PATH, COUNTS_PATH = log_path + ".bulk-lines", log_path + ".bulk-lines.previous", log_path + ".bulk-lines.counts"
for stopping_signal in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
    signal.signal(stopping_signal, signal.SIG_IGN)   # 读到管道收尾才退：写的一方被停之后，已经到手的行照样落盘
for stale_path in (TAIL_PATH, PREVIOUS_TAIL_PATH, COUNTS_PATH):
    if os.path.exists(stale_path):
        os.unlink(stale_path)
log_descriptor = os.open(log_path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o644)
tail_descriptor = None
counts = {prefix: 0 for prefix in BULK_PREFIXES}
kept_with_marks = tail_generation_filled = total_lines = lines_since_forward = 0
log_problem = forward_broken = False
last_line = b""
started = last_forward = time.monotonic()


def write_all(descriptor, data):
    while data:
        data = data[os.write(descriptor, data):]


def write_log(data):
    global log_problem
    try:
        write_all(log_descriptor, data)
    except OSError as error:
        if not log_problem:
            log_problem = True
            print(f"  ! 这一趟的日志 {log_path} 写不下去了（{error}）：后面的输出接着读完、不进日志", file=sys.stderr, flush=True)


def forward(data):
    global forward_broken, last_forward, lines_since_forward
    last_forward = time.monotonic()
    lines_since_forward = 0
    if forward_broken:
        return
    try:
        write_all(1, data)
    except OSError:
        forward_broken = True


def save_counts():
    try:
        with open(COUNTS_PATH + ".writing", "w", encoding="utf-8") as handle:
            for prefix in BULK_PREFIXES:
                handle.write(f"{prefix.decode().strip()}\t{counts[prefix]}\n")
            handle.write(f"kept-with-marks\t{kept_with_marks}\n")
        os.replace(COUNTS_PATH + ".writing", COUNTS_PATH)
    except OSError:
        pass


def keep_in_tail(line):
    global tail_descriptor, tail_generation_filled
    if break_switch == "bulk-tail-dropped":
        return
    try:
        if tail_descriptor is None:
            tail_descriptor = os.open(TAIL_PATH, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o644)
        write_all(tail_descriptor, line)
        tail_generation_filled += 1
        if tail_generation_filled >= tail_generation_lines:
            os.close(tail_descriptor)
            tail_descriptor = None
            os.replace(TAIL_PATH, PREVIOUS_TAIL_PATH)
            tail_generation_filled = 0
            save_counts()
    except OSError:
        pass


def bulk_prefix_of(line):
    global kept_with_marks
    if break_switch == "bulk-lines-kept":
        return None
    for prefix in BULK_PREFIXES:
        if line.startswith(prefix):
            if break_switch != "bulk-lines-with-marks-dropped" and KEPT_MARKS.search(line, len(prefix)):
                kept_with_marks += 1
                return None
            return prefix
    return None


def line_finished(line, already_logged):
    global total_lines, last_line, lines_since_forward
    total_lines += 1
    lines_since_forward += 1
    last_line = line
    prefix = bulk_prefix_of(line) if already_logged == 0 else None
    if prefix is not None:
        counts[prefix] += 1
        keep_in_tail(line)
        return
    write_log(line[already_logged:])
    if FORWARDED_LINE.search(line):
        forward(b"    " + line + (b"" if line.endswith(b"\n") else b"\n"))


def report_activity():
    now = time.monotonic()
    # 这一段时间 cargo 一行都没打就不报：卡住的一趟日志照样不涨，统一任务服务照样判得出卡住（task-server/task_server_lib.py 的 stalled）
    if lines_since_forward == 0 or now - last_forward < activity_seconds or break_switch == "pass-activity-silent":
        return
    shown = last_line.rstrip()[:200].decode("utf-8", "ignore")
    bulk_total = sum(counts.values())
    forward(f"    CRASH_POINT_REPLAY_PASS_ACTIVE seconds={now - started:.0f} lines={total_lines} bulk_lines={bulk_total} last_line={shown}\n".encode())
    save_counts()


pending = b""
logged = 0
while True:
    readable, _, _ = select.select([0], [], [], activity_seconds)
    if readable:
        chunk = os.read(0, 1 << 16)
        if not chunk:
            break
        pending += chunk
        if b"\n" in chunk:
            finished = pending.split(b"\n")
            pending = finished.pop()
            for finished_line in finished:
                line_finished(finished_line + b"\n", logged)
                logged = 0
        # 还没收尾的半行：不会是逐提交的行就先进日志（「test <名> ... 」被别的测试打的字夹住时要到结局打出来才收尾，卡住时靠它看现场）
        if len(pending) > logged and not any(prefix.startswith(pending) or pending.startswith(prefix) for prefix in BULK_PREFIXES) or len(pending) > HELD_LINE_BYTES:
            write_log(pending[logged:])
            logged = len(pending)
            if logged > HELD_LINE_BYTES:
                pending = pending[-4096:]
                logged = len(pending)
    report_activity()
if pending:
    line_finished(pending, logged)
save_counts()
PY_PASS_OUTPUT
)"
if [[ "$crash_point_replay_break" == pass-output-program-in-scratch ]]; then
  printf '%s\n' "$crash_point_replay_pass_output_program" > "$crash_point_replay_scratch_directory/pass-output.py"
fi
# 按条复用两趟的整份输出（文件头「每一趟当场写结局」）：不带特性那一趟、带特性那一趟（排在最后另起的那一趟在它旁边，名字加 .last-pass；
# 并成一份之前带特性那一趟在 .main）
crash_point_replay_plain_pass_log="$crash_point_replay_pass_log_directory/plain-singlefs-verification.log"
crash_point_replay_feature_pass_log="$crash_point_replay_pass_log_directory/features-singlefs-verification.log"
# 弄坏开关 pass-outcome-at-cell-end 下每一趟的结局先记在这里，整格收尾才打（report_pass_outcome、flush_deferred_pass_outcomes）
crash_point_replay_deferred_pass_outcomes="$crash_point_replay_scratch_directory/deferred-pass-outcomes"
# 不带特性那一趟打了 NOT_RUN_ 的测试函数（not_run_markers_of 的写法）与带特性那一趟补判的结论（文件头「多机配置开了 GPU」）；
# 没有分片阶段时单片那一步留下的记号合并那一步接着报，所以按条复用那一趟开跑不清它
crash_point_replay_plain_not_run_file="$crash_point_replay_scratch_directory/plain-not-run"
crash_point_replay_supplement_file="$crash_point_replay_scratch_directory/plain-not-run.supplement"
crash_point_replay_plain_not_run_total=0
: >"$crash_point_replay_plain_not_run_file"
# 单片那一步要现判与拒算的测试函数（一行「<目标根文件><TAB><libtest 全名>」）：按条记库不认的（打了 NOT_RUN_、结果行认不出结局）下一趟照跑，
# 没有分片阶段时合并那一步的核账靠这份清单认出它们单片跑过（crash_point_replay_package_pass_check）
crash_point_replay_shard_ran_file="$crash_point_replay_scratch_directory/shard-ran"
: >"$crash_point_replay_shard_ran_file"

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
# 登了 separate-run= 的用例（文件头「复用」separate-run 那一段）：没有 --item 点名崩溃枚举用例、要跑 crash-case-replay 格的这一趟去掉、逐条报本次未跑；计划、各片、合并三步都走这一段。
# crash_point_replay_report_not_run 定义在后面，这里照它的写法直接打、直接记。弄坏开关 separate-run-unrecognized 下不认这个标记
crash_point_replay_separate_run_count=0
crash_point_replay_separate_run_note=""
if (( ${#crash_point_replay_named_cases[@]} == 0 )) && [[ "$crash_point_replay_break" != separate-run-unrecognized ]] && crash_point_replay_runs_cell "$CRASH_CASE_REPLAY_CELL_NAME"; then
  if ! crash_point_replay_separate_run_listing="$(python3 "$crash_point_replay_admission_module" crash-cases "$ROOT" --separate-run)"; then
    printf '%s\n' "$crash_point_replay_separate_run_listing" | sed 's/^/       /'
    echo "  ✗ .claude/gate.d/stage-inputs.tsv 里 crash-case: 那几行登了 separate-run= 的答不出（上面逐条列出）：判不出这一趟该去掉哪几条单独验证的崩溃枚举用例"
    echo "     → 怎么办：单跑 python3 scripts/gate/admission.py crash-cases <项目根> --separate-run，照 scripts/gate/admission.py 文件头「崩溃枚举用例行」的 separate-run= 改那几行，改到它退 0 为止。"
    exit 1
  fi
  declare -A crash_point_replay_separate_run_reasons=()
  while IFS=$'\t' read -r crash_point_replay_separate_run_key crash_point_replay_separate_run_reason; do
    if [[ -n "$crash_point_replay_separate_run_key" ]]; then crash_point_replay_separate_run_reasons["$crash_point_replay_separate_run_key"]="$crash_point_replay_separate_run_reason"; fi
  done <<< "$crash_point_replay_separate_run_listing"
  if (( ${#crash_point_replay_separate_run_reasons[@]} > 0 )); then
    crash_point_replay_kept_rows=()
    for crash_case_row in "${crash_case_rows[@]}"; do
      crash_point_replay_separate_run_key="${crash_case_row%%$'\t'*}"
      if [[ -z "${crash_point_replay_separate_run_reasons[$crash_point_replay_separate_run_key]+set}" ]]; then
        crash_point_replay_kept_rows+=("$crash_case_row")
        continue
      fi
      crash_point_replay_separate_run_line="本次未跑（独立验证，separate-run=${crash_point_replay_separate_run_reasons[$crash_point_replay_separate_run_key]}）：$crash_point_replay_separate_run_key"
      echo "  ! $crash_point_replay_separate_run_line"
      if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "crash-point-replay.sh 的 $CRASH_CASE_REPLAY_CELL_NAME 格$crash_point_replay_separate_run_line" >> "$GATE_NOT_RUN_FILE"; fi
      crash_point_replay_separate_run_count=$((crash_point_replay_separate_run_count + 1))
    done
    echo "     → 单交（经 task-server/task submit，范围声明写在 submit 前面）：bash .claude/gate.d/crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME --crash-cases-only --item crash-case:<用例名> <项目根>"
    crash_point_replay_separate_run_note="（登了 separate-run= 的 ${crash_point_replay_separate_run_count} 条拆出来单独验证，这一趟没跑）"
    crash_case_rows=("${crash_point_replay_kept_rows[@]}")
  fi
fi

# ── 分片的计划那一步定片数、预先编测试二进制（文件头「分片」plan）──────────
# 一片至少担多少要现判的量，不到就不多分一片（推的，没量过每块与每个测试目标判多久）：一片有固定的开销（排队、起进程、读计划，每条用例各片各录一遍），
#   别的会话量到的那条小用例（compaction-relocating-a-block-shared-with-a-snapshot-segments-shorter-than-ten-clean）每趟 8～9 块，不带 gpu 特性只录那一趟 15.93 秒（任务 14290），
#   几分钟的活拆成五片只多了五份固定开销；取 64 块、8 个测试目标，让这种大小八倍以内的一趟只分一片，量到每块、每个目标判多久之后再改
CRASH_POINT_REPLAY_BLOCKS_PER_SHARD=64
CRASH_POINT_REPLAY_TARGETS_PER_SHARD=8
# 按条复用那一趟（不带特性）起 cargo test 的参数：各片起那一趟（run_checker_package_tests）与计划那一步预先编它（crash_point_replay_prebuild_package_pass_binaries）用同一份，
#   test-targets-parallel.py --binaries 照它核清单是不是这样编的
CRASH_POINT_REPLAY_PACKAGE_PASS_CARGO_ARGUMENTS=(--release --no-fail-fast -p singlefs-verification --lib --tests)

# crash_point_replay_note_stop <哪一趟> <退出码> <日志>：这一趟是不是被停的（文件头「红了接着跑，停了不接着跑」）：cargo 或包着它的那一层退 130、137、143，
#   或日志里 cargo 报测试二进制死于 SIGINT、SIGKILL、SIGTERM（撞内存上限与逐个停进程都是这样；别的信号，如 SIGSEGV、SIGABRT，是测试自己崩了，照红、接着跑）。
#   是的话原因放进 crash_point_replay_stop_reason（先记下的那一处算数），后面要起 cargo 的一概不接着跑。弄坏开关 stopped-pass-runs-the-rest 下一律不认
crash_point_replay_stop_reason=""

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

# make_log_directory：每趟一个新目录（不拿上一趟的输出顶上），放每条崩溃枚举用例的日志与发现日志、覆盖行记录和全绿标记；头一次用到才建，跑完不删
crash_point_replay_log_directory=""

# 登记的崩溃枚举用例与放量分项各落到哪个「测试目标、用例函数」（按条复用的几趟用 --skip 排掉它们）：用没按 --item 筛过的登记取，取不出整道红
crash_point_replay_registered_pairs_file="$crash_point_replay_scratch_directory/registered-functions.tsv"
if registered_test_functions >"$crash_point_replay_registered_pairs_file.raw" 2>"$crash_point_replay_registered_pairs_file.err"; then
  LC_ALL=C sort -u "$crash_point_replay_registered_pairs_file.raw" >"$crash_point_replay_registered_pairs_file"
else
  echo "  ✗ 登记的崩溃枚举用例与放量分项各落到哪个用例函数，取不出来（$(tail -n 2 "$crash_point_replay_registered_pairs_file.err" | tr '\n' ' ' | clip_text 300)）：按条复用那几趟排不掉它们"
  echo "     → 怎么办：单跑 python3 scripts/crash/lib_crash_point_replay_items.py check <根> 看合表与放量入口的 ITEMS 哪里读不了，修好之后重跑这一道。"
  exit 1
fi

# ── 格（次序同文件头格名表，文件头「格的次序」）─────────
red_cells=()
cell_states=()
package_pass_log=""
crash_point_replay_cell_green=0
crash_point_replay_cell_state="绿"
feature_cell_outcome=""
red_cases=(); green_cases=(); reused_cases=(); not_run_cases=(); no_card_cases=()
# 分片的计划那一步（文件头「分片」plan）：上面定了片数的上界；这里算一遍不带特性那一趟的 test-plan 放进计划目录（各片读它）、预先编登记的崩溃枚举用例的测试二进制
#   放进计划目录（各片与合并直接起），再照这一趟要现判的量定片数、写分片的计划（crash_point_replay_write_shard_plan 退 0）
if [[ "$crash_point_replay_shard_phase" == plan ]]; then
  crash_point_replay_store_shared_plan
  crash_point_replay_prebuild_case_binaries "$crash_point_replay_shared_binaries_directory"
  crash_point_replay_prebuild_package_pass_binaries "${crash_point_replay_shared_binaries_directory:+$crash_point_replay_shared_binaries_directory/package-pass}"
  crash_point_replay_sample_cargo_count_note "计划"
  crash_point_replay_decide_shard_count
  crash_point_replay_write_shard_plan "$crash_point_replay_shard_plan_count" \
    "$crash_point_replay_shard_plan_origin；${#crash_point_replay_named_cases[@]} 条点名的崩溃枚举用例（0 是全部登记的）"
fi
if [[ "$crash_point_replay_shard_phase" == shard ]]; then
  # 判绿收尾的片留着计划那一步放的测试二进制给合并用，别的收尾删（crash_point_replay_drop_shared_binaries）
  if crash_point_replay_run_shard_step; then
    crash_point_replay_shard_step_green=1
    exit 0
  else
    exit $?
  fi
fi
# 没有分片阶段（文件头「分片」）：这一件里依次走计划、单片（第 0/1 片）、合并。单片调上面同一个 crash_point_replay_run_shard_step，合并就是下面照 merge 那一步往下走的那一段，不另写不分片的路。
#   单片退非 0 整道就退它的码、不起合并（同服务：有红的片撤掉合并）；单片留下的覆盖行存不成的记号清掉，合并那一步照键取覆盖行、自己再记。
#   弄坏开关 no-phase-runs-as-merge 下不做计划与单片，没有分片阶段就当合并走
if [[ -z "$crash_point_replay_shard_phase" && "$crash_point_replay_break" != no-phase-runs-as-merge ]]; then
  echo "── 没有 TASK_SERVER_SHARD_PHASE：这一件里自己走计划、单片、合并三步"
  if crash_point_replay_runs_cell "$CRASH_CASE_REPLAY_CELL_NAME"; then
    # 样本根的 .gate-shard-count（一个正整数，文件头「样本档」）：照服务分片那样走 N 片、预先编测试二进制，样本据此演计划编一回、各片与合并不编；
    #   只在样本根上认，真跑照旧 1 片、不预先编（同一个进程里单片与合并用同一份编译目录，没有各片各编一遍的事）
    crash_point_replay_alone_shard_count=1
    if [[ "$crash_point_replay_sample_mode" == 1 && -f .gate-shard-count ]]; then
      crash_point_replay_alone_shard_count="$(sed 's/#.*//' .gate-shard-count | tr -d '[:space:]')"   # # 之后与空白不算（同 .gate-items）
      if [[ ! "$crash_point_replay_alone_shard_count" =~ ^[1-9][0-9]?$ ]]; then
        echo "  ✗ crash-point-replay.sh：样本根 $ROOT 的 .gate-shard-count 写的是「$crash_point_replay_alone_shard_count」，不是 1 到 99 的整数"
        echo "     → 怎么办：写这个样本要演几片（例 2）"
        exit 2
      fi
      echo "  · 计划：$crash_point_replay_alone_shard_count 片（样本根的 .gate-shard-count 点名；照服务分片的计划那一步预先编登记的崩溃枚举用例的测试二进制，各片与合并直接起）"
      crash_point_replay_shared_binaries_directory="$crash_point_replay_scratch_directory/shard-plan/test-binaries"
      mkdir -p -- "$crash_point_replay_scratch_directory/shard-plan"
      crash_point_replay_prebuild_case_binaries "$crash_point_replay_shared_binaries_directory"
    else
      echo "  · 计划：1 片（第 0/1 片，SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/1）"
    fi
    # 计划这一步也算一遍不带特性那一趟的 test-plan（放草稿目录里），单片读它；合并照当时的源码现算
    crash_point_replay_shared_plan_directory="$crash_point_replay_scratch_directory/shared-plan"
    crash_point_replay_store_shared_plan
    # 样本根点名了几片的（上面）另把按条复用那一趟的测试二进制编好放进同一处；真跑 1 片时没有计划那一步放的目录，不编
    crash_point_replay_prebuild_package_pass_binaries "${crash_point_replay_shared_binaries_directory:+$crash_point_replay_shared_binaries_directory/package-pass}"
    crash_point_replay_sample_plan_count_note "计划"
    crash_point_replay_sample_cargo_count_note "计划"
    crash_point_replay_shard_phase=shard
    crash_point_replay_shard_count="$crash_point_replay_alone_shard_count"
    for (( crash_point_replay_shard_index = 0; crash_point_replay_shard_index < crash_point_replay_shard_count; crash_point_replay_shard_index++ )); do
      if crash_point_replay_run_shard_step; then crash_point_replay_alone_shard_exit=0; else crash_point_replay_alone_shard_exit=$?; fi
      (( crash_point_replay_alone_shard_exit == 0 )) || exit "$crash_point_replay_alone_shard_exit"
      if (( crash_point_replay_shard_count == 1 )); then
        crash_point_replay_sample_plan_count_note "单片"
        crash_point_replay_sample_cargo_count_note "单片"
      else
        crash_point_replay_sample_cargo_count_note "第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片"
      fi
    done
    crash_point_replay_shard_index=0
    crash_point_replay_shard_count=1
  else
    echo "  · 计划：0 片（这一趟没点 $CRASH_CASE_REPLAY_CELL_NAME 格），点了的格在合并那一步跑"
  fi
  crash_point_replay_shard_phase=merge
  crash_point_replay_flow_records_unavailable=""
  crash_point_replay_flow_records_failed=""
  echo "── 合并：九格照常判，$CRASH_CASE_REPLAY_CELL_NAME 格核账、一块不现判"
fi
if crash_point_replay_runs_cell "$CRASH_CASE_REPLAY_CELL_NAME"; then
  package_pass_report=""
  package_pass_plan_red=0
  make_log_directory
  if run_package_test_functions; then package_pass_green=1; else package_pass_green=0; fi
  crash_point_replay_sample_plan_count_note "合并"
  crash_cases_green=0
  if crash_point_replay_package_pass_holds_cases_back "$package_pass_green"; then
    :
  elif run_registered_crash_cases; then
    crash_cases_green=1
  else
    # 按块判定红了是真红：记库那一步 prune-server 不可用之后块又红的，照红算（crash_point_replay_red_regardless_cells）
    crash_point_replay_red_regardless_cells+=("$CRASH_CASE_REPLAY_CELL_NAME")
  fi
  crash_point_replay_sample_cargo_count_note "合并"
  if (( package_pass_green && crash_cases_green )) && package_pass_reuse_held; then
    (( package_pass_ran )) && package_pass_log="$crash_point_replay_plain_pass_log"
    crash_point_replay_cell_green=1
  else
    if (( ! package_pass_green && ${#red_cases[@]} > 0 )); then
      echo "  ✗ $CRASH_CASE_REPLAY_CELL_NAME 两部分都红：不带特性那一趟判红，${#red_cases[@]} 条崩溃枚举用例判红（${red_cases[*]}）"
      echo "     → 怎么办：两部分各照上面自己那一句「→ 怎么办」改；改完只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME"
    fi
    # 这一格红了：不带特性那一趟的本次未跑不补判、不报（红句已经点了出路）
    : >"$crash_point_replay_plain_not_run_file"
    red_cells+=("$CRASH_CASE_REPLAY_CELL_NAME")
  fi
  flush_deferred_pass_outcomes
  if (( crash_point_replay_cell_green )); then crash_point_replay_announce_cell "$CRASH_CASE_REPLAY_CELL_NAME" "绿"
  else crash_point_replay_announce_cell "$CRASH_CASE_REPLAY_CELL_NAME" "$(red_cell_state "$CRASH_CASE_REPLAY_CELL_NAME")"; fi
fi
[[ "$crash_point_replay_break" == last-pass-before-the-other-cells ]] && crash_point_replay_run_feature_cell
# 产品 GPU 包的真显卡对拍（文件头「product-gpu-real-card-tests 格」）：与别的两趟互不依赖，不进覆盖、ignore、0 次那几格的行
product_gpu_cell_outcome=""
product_gpu_cell_state=""
if crash_point_replay_runs_cell "$PRODUCT_GPU_REAL_CARD_CELL_NAME"; then
  if crash_point_replay_stopped_before_cell "$PRODUCT_GPU_REAL_CARD_CELL_NAME"; then
    product_gpu_cell_outcome="stopped"
  elif run_product_gpu_real_card_cell; then
    product_gpu_cell_outcome="ran"
    crash_point_replay_announce_cell "$PRODUCT_GPU_REAL_CARD_CELL_NAME" "$product_gpu_cell_state"
  else
    product_gpu_cell_outcome="red"
    red_cells+=("$PRODUCT_GPU_REAL_CARD_CELL_NAME")
    crash_point_replay_announce_cell "$PRODUCT_GPU_REAL_CARD_CELL_NAME" "$(red_cell_state "$PRODUCT_GPU_REAL_CARD_CELL_NAME")"
  fi
fi
# 只读树的三格（文件头「格的次序」）：不等带特性那一趟
crash_point_replay_run_audit_cells "${CRASH_AUDIT_READ_ONLY_CELLS[@]}"
crash_point_replay_run_feature_cell
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
elif [[ "$feature_cell_outcome" == stopped ]]; then
  cell_states+=("$FEATURE_GATED_TESTS_CELL_NAME：本次未跑（这一趟被停了）")
fi
if [[ "$product_gpu_cell_outcome" == ran ]]; then
  cell_states+=("$PRODUCT_GPU_REAL_CARD_CELL_NAME：$product_gpu_cell_state")
elif [[ "$product_gpu_cell_outcome" == red ]]; then
  cell_states+=("$PRODUCT_GPU_REAL_CARD_CELL_NAME：$(red_cell_state "$PRODUCT_GPU_REAL_CARD_CELL_NAME")")
elif [[ "$product_gpu_cell_outcome" == stopped ]]; then
  cell_states+=("$PRODUCT_GPU_REAL_CARD_CELL_NAME：本次未跑（这一趟被停了）")
fi
# 覆盖、ignore、0 次三格读上面两趟与每条用例的行，排在它们后面；点了 --check 的只跑点名的
flow_cells_wanted=0
for crash_audit_cell in "${CRASH_AUDIT_FLOW_CELLS[@]}"; do crash_point_replay_runs_cell "$crash_audit_cell" && flow_cells_wanted=1; done
if (( flow_cells_wanted )) && [[ -z "$crash_point_replay_stop_reason" ]]; then make_log_directory; prepare_flow_records; fi
crash_point_replay_run_audit_cells "${CRASH_AUDIT_FLOW_CELLS[@]}"
# 逐格汇总与红格清单的次序不跟着跑的次序走（文件头「格的次序」）：先是起 cargo 的三格，再是覆盖、ignore、0 次、必红世界、旋钮、来历表；只重跑红格的 --check 照这个次序写
[[ "$crash_point_replay_break" == crash-audit-red-ignored ]] && crash_audit_red=()   # 弄坏开关：出口 4 那几格红了也不算进整道
for crash_audit_cell in "${CRASH_AUDIT_CELL_NAMES[@]}" "$CRASH_CASE_PROVENANCE_CELL_NAME"; do
  for crash_audit_state in ${crash_audit_states[@]+"${crash_audit_states[@]}"}; do
    [[ "${crash_audit_state%%：*}" == "$crash_audit_cell" ]] && cell_states+=("$crash_audit_state")
  done
  crash_point_replay_cell_in "$crash_audit_cell" ${crash_audit_red[@]+"${crash_audit_red[@]}"} && red_cells+=("$crash_audit_cell")
done
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
  (( ${#no_card_cases[@]} > 0 )) && echo "     另有 ${#no_card_cases[@]} 条崩溃枚举用例本次未跑（观察者交回的盘取不到卡）：${no_card_cases[*]}；$(crash_point_replay_observer_images_howto)"
  exit 1
fi
# 登记的崩溃枚举用例有观察者交回的盘取不到卡而记了本次未跑的（文件头「显卡」）：别的格没有红，整道退 78（不是红，也不算通过），不写 all-green
gpu_not_run_cases=(${no_card_cases[@]+"${no_card_cases[@]}"})
if (( ${#gpu_not_run_cases[@]} > 0 )); then
  echo "  ✗ ${#gpu_not_run_cases[@]} 条崩溃枚举用例本次未跑（不是红，也不算通过）：${gpu_not_run_cases[*]}"
  if (( ${#no_card_cases[@]} > 0 )); then
    echo "     → 怎么办：$(crash_point_replay_observer_images_howto)；收了之后只重跑这几条：bash .claude/gate.d/crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME --crash-cases-only$(printf ' --item %s' "${no_card_cases[@]}")"
  fi
  exit 78
fi
crash_point_replay_pass_logs_disposable=1   # 判绿收尾：每一趟 cargo 的整份输出不用留（文件头「每一趟当场写结局」；每条用例的整份输出另在跑完不删的日志目录里，不受这一行管）
# 缩过域的这一趟（环境里设了展开深度变量，引擎只往小缩）：判绿也不算门禁通过，不写 all-green；点没点范围都写出设了的变量与值
crash_point_replay_narrowed=""
# 这份名单与 scripts/gate/admission.py 的 CRASH_CASE_NARROWING_VARIABLES（crash-case-judge 判 exhaustive= 那一判读的）是同一份，两处一起改
for crash_point_replay_narrowing_variable in SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO; do
  [[ -n "${!crash_point_replay_narrowing_variable+x}" ]] || continue
  crash_point_replay_narrowed+="${crash_point_replay_narrowed:+、}$crash_point_replay_narrowing_variable=${!crash_point_replay_narrowing_variable}"
done
[[ "$crash_point_replay_break" == narrowed-run-writes-all-green ]] && crash_point_replay_narrowed=""   # 弄坏开关：不看缩域变量
if [[ -n "$crash_point_replay_narrowed" ]]; then
  echo "  · 缩过域、不算门禁通过：$crash_point_replay_narrowed（只往小缩，大于引擎常量的不放宽）；不写 all-green，scripts/gate/apply-writer-patch.py 数上次崩溃验证全绿以来打了几个补丁时这一趟不算全绿"
fi
# 不指定范围（没给 --check，也没给 --item）而且没缩过域判绿的这一趟：日志目录里留一份 all-green（只记这一趟绿过，scripts/gate/apply-writer-patch.py 数「上次崩溃验证全绿以来打了几个补丁」用；不是复用的标记）
if (( ! crash_point_replay_range_given )) && [[ -z "$crash_point_replay_narrowed" ]] && [[ "$crash_point_replay_break" != all-green-not-written ]] && (( crash_point_replay_cell_green )) && [[ -n "$crash_point_replay_log_directory" ]]; then
  if printf '%s\n' "$(TZ=Asia/Tokyo date +%F) ${#crash_case_rows[@]} 条崩溃枚举用例全部判绿${crash_point_replay_separate_run_note}" > "$crash_point_replay_log_directory/all-green"; then
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
