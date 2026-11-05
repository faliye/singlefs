# 阶段同步第二段：逐行判 F3-F5,F11（批次 12，阶段 crash-slice-false-resume）

- 候选表：`/home/fy5090/.local/share/singlefs/work/cpr-sync-facts-sweep/candidates.tsv`；本份只判 F3（21 行）、F4（39 行）、F5（19 行）、F11（68 行），共 147 行（量的：`awk -F'\t' '$1=="F3"||$1=="F4"||$1=="F5"||$1=="F11"' candidates.tsv | wc -l` 得 147）。
- 基准 `59ce5a865`，结束 `7ac20c0204a7587b580a8eb2d78c7efead4e873f`；每行都用 `git show 结束提交:载体路径` 读结束那一版的原行，不读主工作区（主工作区只在核「C786 已挪进已还清」时对照）。我这 147 行的载体文件里 `<<<<<<<` 标记 0 处（量的：逐个 `git show … | grep -c '^<<<<<<<'`），冲突标记只在 crates/ 下三份与 scripts/hosts/ 下三份，不涉及。
- 判法：事件句看「把旧说法换成新的，这句还是不是真话」；只提到检索词、说的不是这件事实的判不相干。

## 摘要

- 结果（量的：对本份表格按组、判定数行）：要改 6 行（F3 的 4 行、F11 的 2 行），事件句不改 104 行，不相干 37 行，要补 0 行、要人看 0 行；反向补 7 行（M1 要补，M2-M7 要改，都是候选表没罩到的同源句）。
- 要改的 6 行：
  - C786 三语开着的行（`.claude/kb/checks-owed.md:502`、`.en.md:502`、`.ja.md:502`）：结束提交那一版还挂在开着的表里写「各数一次」，主工作区三语都已挪进已还清（第 933 行，量的），提交时带上即可。
  - `.claude/kb/milestone/05-fifth-txn.md:153`（八·3）：「做什么」列还要做「双机导入的块只数一次」、欠账列还列 C786，该去掉。
  - C564（`.claude/kb/checks-owed.md:348`）写「已登记 34 条」、C914（`.claude/kb/checks-owed.md:589`）写「36 条登记用例」：结束提交那一版 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行 51 条（量的：`git show 结束提交:… | grep -c '^crash-case:'`），基准那一版 47 条；同文件 `.claude/kb/verification-build.md` 第 6 行已写「条数用 grep -c … 现数，正文不抄数」，改成同一写法最稳。
- 候选表没罩到的 7 行见「逐行判定」末尾 M1-M7：C564、C914 的英日行（检索词只有中文，没搜到）、05-fifth-txn 八·3 的英日行、04-fourth-txn 没记这一阶段的新用例与假续跑修复。
- 带出一条旁证：`.claude/kb/milestone/03-third-txn.md:226`（行 44）里的 `gpu_judge_units_fingerprint_of_this_process` 不带特性时没人调，基准与结束那一版的该函数都已是 `#[cfg(any(feature = "gpu", test))]`（量的：两版 crash_amplification.rs 的该函数上一行），这一行的「开着」可能早就过时，但与这一阶段无关，判不相干，交主 agent 另看。

## 交实现员

0 行。载体在 `crates/` 下的候选一行都没有（候选表 F3、F4、F5、F11 的载体都是 kb、规则、records、README、skill 与 agent 定义），本份判要改要补的行也没有 `crates/` 目标。数行的命令：`grep -E '^\| (F3|F4|F5|F11|M[0-9]+) \| crates/' <本报告> | grep -cE '\| (要改|要补) \|'`，输出见文末。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F3 | .claude/kb/checks-owed.en.md:502 | 要改 | 结束提交那一版 C786 仍在开着的表里、写「各数一次」，而 `imported_blocks` 已按块键去重（库单测 the_imported_blocks_count_the_blocks_found_in_any_library_and_not_their_conclusions 把同一块存进两份库、断言找到三块），调度记录已判还清；改法：把这一行从开着的表挪进「已还清」，写成「`imported_blocks` 改成数块、同一块只数一次」（主工作区该文件第 933 行已是这个形态，提交时带上）（原话「imported_block」） |
| F3 | .claude/kb/checks-owed.ja.md:502 | 要改 | 结束提交那一版 C786 仍在开着的表里、写「各数一次」，而 `imported_blocks` 已按块键去重（库单测 the_imported_blocks_count_the_blocks_found_in_any_library_and_not_their_conclusions 把同一块存进两份库、断言找到三块），调度记录已判还清；改法：把这一行从开着的表挪进「已还清」，写成「`imported_blocks` 改成数块、同一块只数一次」（主工作区该文件第 933 行已是这个形态，提交时带上）（原话「imported_block」） |
| F3 | .claude/kb/checks-owed.md:502 | 要改 | 结束提交那一版 C786 仍在开着的表里、写「各数一次」，而 `imported_blocks` 已按块键去重（库单测 the_imported_blocks_count_the_blocks_found_in_any_library_and_not_their_conclusions 把同一块存进两份库、断言找到三块），调度记录已判还清；改法：把这一行从开着的表挪进「已还清」，写成「`imported_blocks` 改成数块、同一块只数一次」（主工作区该文件第 933 行已是这个形态，提交时带上）（原话「同一块各数一次」） |
| F3 | .claude/kb/milestone/05-fifth-txn.md:153 | 要改 | 八·3 的「做什么」列去掉「，双机导入的块只数一次」，「压着的条款与欠账」列去掉「C786（按计划导入几份库同一块各数一次）、」（C786 已判还清、挪进已还清表）；英日同一行同改，候选表没罩到，见 M6、M7（原话「导入的块只数一次；长跑的项按」） |
| F3 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:54 | 事件句不改 | 2026-09-28 那几趟实跑日志里的旧口径原话（导入趟的结论块数），说的是那一趟打了什么；换成新口径这句就不真，不改（原话「/ 复用验证过：同库同代码重」） |
| F3 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:55 | 事件句不改 | 2026-09-28 那几趟实跑日志里的旧口径原话（导入趟的结论块数），说的是那一趟打了什么；换成新口径这句就不真，不改（原话「imported_block」） |
| F3 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:61 | 事件句不改 | 2026-09-28 那几趟实跑日志里的旧口径原话（导入趟的结论块数），说的是那一趟打了什么；换成新口径这句就不真，不改（原话「/ 全域起跑 / 前置十步（」） |
| F3 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:63 | 事件句不改 | 2026-09-28 那几趟实跑日志里的旧口径原话（导入趟的结论块数），说的是那一趟打了什么；换成新口径这句就不真，不改（原话「/ 四种跑法在 18 域全部」） |
| F3 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:64 | 事件句不改 | 2026-09-28 那几趟实跑日志里的旧口径原话（导入趟的结论块数），说的是那一趟打了什么；换成新口径这句就不真，不改（原话「/ 分片与 GPU 跑通；用」） |
| F3 | records/2026-10-01-删除与并发事务.md:1550 | 事件句不改 | 2026-10-11 代码轮判决的经过（P5-2 打中），是那一次发生的事（原话「imported_block」） |
| F3 | records/2026-10-01-删除与并发事务.md:1570 | 事件句不改 | 写回时记各判决第四节欠账的经过，是那一次的操作记录（原话「imported_block」） |
| F3 | records/2026-10-01-删除与并发事务.md:1589 | 事件句不改 | 登记 C786 等欠账那一笔的经过，是那一次的操作记录（原话「同一块各数一次）。没进 ch」） |
| F3 | records/2026-10-13-复用键与全仓改名.md:769 | 事件句不改 | 2026-10-13 判决里核对日志 imported_blocks=8 的经过，是那一次的核对（原话「imported_block」） |
| F3 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:12 | 事件句不改 | 调度记录证据表：2026-10-27 那趟日志第 44 行打出的原样数，是那一趟的证据（原话「imported_block」） |
| F3 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:14 | 事件句不改 | 同一张证据表：本机导入那一趟打出的旧口径数（53680 是结论块数），是那一趟的证据（原话「imported_block」） |
| F3 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:86 | 事件句不改 | 依赖表 D7 行，是这一批开头发现问题时的原样记录（「未另核」是当时的核验状态）（原话「imported_block」） |
| F3 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:166 | 事件句不改 | 批次 4 交回当时的观察，「仍数结论块」说的是改之前，并进批次 7 的安排也是那一次的（原话「imported_block」） |
| F3 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:178 | 事件句不改 | 批次安排表 7b 行，写的是当时派什么活与状态（批次 5 已打），是那一次的安排（原话「imported_block」） |
| F3 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:193 | 事件句不改 | 批次 7 交回之二的补丁描述，写的就是改成之后的样子（三行同为块数），是那一次交回的事（原话「imported_block」） |
| F3 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:199 | 事件句不改 | 批次 4 交回里对 C786 的当时处理意见（算不算还清等验证段证红之后定）；工作区调度记录的「验证段收尾」一节已判还清，这句是当时的状态，不改（原话「同一块各数一次）：改成数块之」） |
| F3 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:230 | 事件句不改 | 依赖表条款逐条有实现的核对结论，是批次验收那一次的核对（原话「imported_block」） |
| F4 | .claude/kb/milestone/03-third-txn.en.md:226 | 不相干 | 这一行说的是 check.sh 的 clippy 只跑全特性、门禁看不见不带特性才暴露的 dead_code，函数名只是例子，不涉及哪条路线取哪一组判器组（原话「gpu_judge_unit」） |
| F4 | .claude/kb/milestone/03-third-txn.ja.md:226 | 不相干 | 这一行说的是 check.sh 的 clippy 只跑全特性、门禁看不见不带特性才暴露的 dead_code，函数名只是例子，不涉及哪条路线取哪一组判器组（原话「gpu_judge_unit」） |
| F4 | .claude/kb/milestone/03-third-txn.md:226 | 不相干 | 这一行说的是 check.sh 的 clippy 只跑全特性、门禁看不见不带特性才暴露的 dead_code，函数名只是例子，不涉及哪条路线取哪一组判器组（原话「gpu_judge_unit」） |
| F4 | .claude/kb/verification-build.md:6 | 不相干 | 说的是 build.rs 经 unit-graph 按 judging_units_entries.rs 算判器组单元键，不说哪条判器路线取哪一组；同一行的登记条数也写「正文不抄数」（原话「**现状**：`crates」） |
| F4 | .claude/rules/verification.md:37 | 不相干 | 说的是块键与判器组单元键怎么组成、带不带 gpu 特性编的两趟键相同，不说运行时哪条路线取 GPU 判器组，这件事实不改它（原话「/ 崩溃放量全量 / `ba」） |
| F4 | records/2026-10-13-复用键与全仓改名.md:662 | 事件句不改 | 2026-10-13 一批交回的核对项（CPU 判器组四入口键相同），是那一次的核对（原话「crates 第二轮交回（`」） |
| F4 | records/2026-10-13-复用键与全仓改名.md:703 | 事件句不改 | 单元图放宽 blanket 之后的代价说明与定案，是那一次的事（原话「crates 第四轮 A 交」） |
| F4 | records/2026-10-13-复用键与全仓改名.md:869 | 事件句不改 | 「甲」代码改完当时的改动描述（GPU 判器拒算时记在占位下等），是那一次的经过（原话「判器组」） |
| F4 | records/2026-10-13-复用键与全仓改名.md:882 | 事件句不改 | 一趟全量日志里的 CRASH_LIBRARY_KEY_CHANGED 行与主 agent 当时的核对，是那一趟的事（原话「CRASH_LIBRARY_」） |
| F4 | records/2026-10-13-复用键与全仓改名.md:944 | 事件句不改 | 回答用户「数据和主机也不绑定吧」的当时说明，是那一次的问答（原话「判器组键只由函数单元闭包、工」） |
| F4 | records/2026-10-13-复用键与全仓改名.md:971 | 事件句不改 | 一次交用户定的修法说明（抬 WGSL 上限会换判器组的键），是当时的判断（原话「- 第 12 条红的调查交回」） |
| F4 | records/2026-10-13-复用键与全仓改名.md:1015 | 事件句不改 | 批次安排表里「全量重跑」一项的当时安排，是那一次的计划（原话「判器组与观察者录入键都变，镜」） |
| F4 | records/2026-10-13-复用键与全仓改名.md:1029 | 事件句不改 | 批 3 起跑的记录，是那一次起跑的事（原话「判器组键与观察者录入键都换了」） |
| F4 | records/2026-10-13-复用键与全仓改名.md:1030 | 事件句不改 | 重起后核对 GPU 判器组键的经过，并含主 agent 对先前说法的自我更正，是那一趟的事（原话「CRASH_LIBRARY_」） |
| F4 | records/2026-10-13-复用键与全仓改名.md:1041 | 事件句不改 | 一批改动的描述与验证次序，是那一次的操作（原话「判法与改法（主 agent 」） |
| F4 | records/2026-10-13-复用键与全仓改名.md:1043 | 事件句不改 | 三条单测改完的验证记录，是那一次的验证（原话「gpu_judge_unit」） |
| F4 | records/2026-10-13-复用键与全仓改名.md:1044 | 事件句不改 | 日志核对的经过，是那一次的核对（原话「探针流改到 mkfs 过的池」） |
| F4 | records/2026-10-13-里程碑四预开发.md:520 | 事件句不改 | 里程碑四预开发记录里对 dead_code 归属的核对，是那一次的事（原话「gpu_judge_unit」） |
| F4 | records/2026-10-13-里程碑四预开发.md:654 | 事件句不改 | 对「广谱变异」一路报告的评判，是那一次的判断（原话「判器组单元键（块判定键就是这」） |
| F4 | records/2026-10-13-里程碑四预开发.md:693 | 事件句不改 | C821 并入修红批的经过，是那一次的事（原话「判器组单元键随之变。整体再验」） |
| F4 | records/2026-10-13-里程碑四预开发.md:1143 | 事件句不改 | 主 agent 对实现员设计问题的当时评判（认或不认），是那一次的评判（原话「判器组的合指纹（准确优先），」） |
| F4 | records/2026-10-13-里程碑四预开发.md:1159 | 事件句不改 | 同上，判法键取法的当时评判（「不带 gpu 编法下不进键——不认」），是那一次的评判（原话「判器组不进键——不认，开了 」） |
| F4 | records/2026-10-13-里程碑四预开发.md:1182 | 事件句不改 | 2026-10-26 判器键实现员交回与主 agent 的评判，是那一次的事（原话「判器组一律取产品行为视图的键」） |
| F4 | records/2026-10-18-里程碑三收尾与近期欠账.md:91 | 事件句不改 | b6 核 clippy 红归属的经过，是那一次的事（原话「gpu_judge_unit」） |
| F4 | records/2026-10-18-里程碑三收尾与近期欠账.md:107 | 事件句不改 | 重编退 101 的原因与当时的修法，是那一次的事（原话「gpu_judge_unit」） |
| F4 | records/2026-10-18-里程碑三收尾与近期欠账.md:266 | 事件句不改 | C821 补丁打进主工作区的经过，是那一次的事（原话「判器组单元键，旧结论不再按键」） |
| F4 | records/2026-10-18-里程碑三收尾与近期欠账.md:797 | 事件句不改 | 一条用例当时另报 KEY_REFUSED 的现象与处理，是那一趟的事（原话「KEY_REFUSED it」） |
| F4 | records/2026-10-18-里程碑三收尾与近期欠账.md:804 | 事件句不改 | 当时对内存撞限的处理决定，是那一次的决定（原话「- mkfs 快档撞内存的调」） |
| F4 | records/2026-10-18-里程碑三收尾与近期欠账.md:922 | 事件句不改 | 快档当时报 CRASH_LIBRARY_KEY_CHANGED 的现象，是那一趟的事（原话「CRASH_LIBRARY_」） |
| F4 | records/2026-10-18-里程碑三收尾与近期欠账.md:1304 | 事件句不改 | 交回里当时列的七条待认项，是那一次交回的事（原话「判器组拒算时观察者判段整趟报」） |
| F4 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:168 | 事件句不改 | 当时被杀的现象与处理，是那一趟的事（原话「CRASH_LIBRARY_」） |
| F4 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:316 | 事件句不改 | 对编译目录与 build.rs 耗时的当时分析，是那一次的事（原话「- 第 20 批调查员交回（」） |
| F4 | records/2026-10-24-checker审计复核修复.md:55 | 事件句不改 | 2026-10-24 实现员当时的推断（判器组键会变的入口），是那一次的事（原话「判器组」） |
| F4 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:12 | 事件句不改 | 调度记录证据表：2026-10-27 那趟日志的原样行，是那一趟的证据（原话「CRASH_LIBRARY_」） |
| F4 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:36 | 事件句不改 | 调度记录根因表：定位到 GPU 判器组键被当「这一趟的键」的那一次读代码的结论，是当时的发现（原话「CRASH_LIBRARY_」） |
| F4 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:85 | 事件句不改 | 依赖表 D6 行，是这一批开头发现问题时的原样记录（「未另核」是当时的核验状态）（原话「判器组拒算时 CPU 路线也」） |
| F4 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:197 | 事件句不改 | 批次 4 交回当时列的待办（给定状态引擎那一处并进下一批），后来由 8b 做完，这句是当时的状态（原话「判器组不看路线：给定状态引擎」） |
| F4 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:219 | 事件句不改 | 批次安排表 8b 行，写的是当时派什么活，是那一次的安排（原话「判器组；数块函数遇旧布局导入」） |
| F4 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:225 | 事件句不改 | 批次 8b 交回的描述，写的就是改成之后的样子（按 routes.crash_judge 取），是那一次交回的事（原话「判器组」） |
| F5 | .claude/kb/experiments/156-alloc-basis-の四つの分岐に伴うコストの数値-一部実施済み.ja.md:42 | 不相干 | 说的是 e156 装置 verdict 函数 panic 的消息只点名 STAGE 与 SHARE 两个变量、真因是没带 gpu 特性编，与 admission.py 零块判定、crash_case_launch 清变量无关（原话「SINGLEFS_CRASH」） |
| F5 | .claude/kb/experiments/156-alloc-basis-の四つの分岐に伴うコストの数値-一部実施済み.ja.md:270 | 不相干 | 说的是 e156 装置 verdict 函数 panic 的消息只点名 STAGE 与 SHARE 两个变量、真因是没带 gpu 特性编，与 admission.py 零块判定、crash_case_launch 清变量无关（原话「SINGLEFS_CRASH」） |
| F5 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:42 | 不相干 | 说的是 e156 装置 verdict 函数 panic 的消息只点名 STAGE 与 SHARE 两个变量、真因是没带 gpu 特性编，与 admission.py 零块判定、crash_case_launch 清变量无关（原话「SINGLEFS_CRASH」） |
| F5 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:270 | 不相干 | 说的是 e156 装置 verdict 函数 panic 的消息只点名 STAGE 与 SHARE 两个变量、真因是没带 gpu 特性编，与 admission.py 零块判定、crash_case_launch 清变量无关（原话「SINGLEFS_CRASH」） |
| F5 | .claude/kb/experiments/156-cost-figures-for-the-four-alloc-basis-forks.en.md:42 | 不相干 | 说的是 e156 装置 verdict 函数 panic 的消息只点名 STAGE 与 SHARE 两个变量、真因是没带 gpu 特性编，与 admission.py 零块判定、crash_case_launch 清变量无关（原话「SINGLEFS_CRASH」） |
| F5 | .claude/kb/experiments/156-cost-figures-for-the-four-alloc-basis-forks.en.md:270 | 不相干 | 说的是 e156 装置 verdict 函数 panic 的消息只点名 STAGE 与 SHARE 两个变量、真因是没带 gpu 特性编，与 admission.py 零块判定、crash_case_launch 清变量无关（原话「SINGLEFS_CRASH」） |
| F5 | .claude/kb/verification-build.en.md:220 | 不相干 | 说的是 SINGLEFS_CRASH_AMPLIFICATION_STAGE 是选段开关（two_stages、RunStage），crash_case_launch 另清它不改它是什么（原话「/ Recording st」） |
| F5 | .claude/kb/verification-build.ja.md:220 | 不相干 | 说的是 SINGLEFS_CRASH_AMPLIFICATION_STAGE 是选段开关（two_stages、RunStage），crash_case_launch 另清它不改它是什么（原话「SINGLEFS_CRASH」） |
| F5 | .claude/kb/verification-build.md:267 | 不相干 | 说的是 SINGLEFS_CRASH_AMPLIFICATION_STAGE 是选段开关（two_stages、RunStage），crash_case_launch 另清它不改它是什么（原话「SINGLEFS_CRASH」） |
| F5 | records/2026-09-24-里程碑二收尾调度.md:215 | 事件句不改 | 2026-09-24 判决复跑 admission.py 207 格的经过，D2 入口多了 crash_case_launch 是那一次的核对（原话「/ G1 交回（2026-0」） |
| F5 | records/2026-10-11-崩溃放量两段与脚本审计.md:49 | 事件句不改 | 2026-10-11 崩溃放量两段协议落地的经过，是那一次落地的事（原话「SINGLEFS_CRASH」） |
| F5 | records/2026-10-11-崩溃放量两段与脚本审计.md:51 | 事件句不改 | 2026-10-11 崩溃放量两段协议落地的经过，是那一次落地的事（原话「- 2026-10-11：课」） |
| F5 | records/2026-10-13-复用键与全仓改名.md:283 | 事件句不改 | 2026-10-13 调查第二台录入段没录的原因（第二台写死 STAGE=judge），是那一次的发现（原话「SINGLEFS_CRASH」） |
| F5 | records/2026-10-13-复用键与全仓改名.md:841 | 事件句不改 | 一趟全量的门禁判红：日志读到 0 行 CRASH_CASE_BLOCKS 行（行数为零，与块数合计为零是两件事），是那一趟的事（原话「crash-case-blo」） |
| F5 | records/2026-10-13-复用键与全仓改名.md:1048 | 事件句不改 | 派发记录（新模式 crash-case-blocks --ran），是那一次的派发（原话「crash-case-blo」） |
| F5 | records/2026-10-13-复用键与全仓改名.md:1052 | 事件句不改 | 工具实现员交回的描述，是那一次交回的事（原话「crash-case-blo」） |
| F5 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:83 | 事件句不改 | 调度记录依赖表 D5 行，是这一批开头发现问题时的原样记录（零块当罩住：假绿）（原话「judge_crash_ca」） |
| F5 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:120 | 事件句不改 | 批次 4 交回对 admission.py 的修补描述，写的就是改成之后的样子，是那一次交回的事（原话「crash_case_lau」） |
| F5 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:121 | 事件句不改 | 主 agent 现查 rc=3、4、4 与自证 288 格的结果，是那一次的核对（原话「crash-case-blo」） |
| F11 | .claude/agents/implementation-writer.md:49 | 不相干 | 说的是怎么向主 agent 报要登记的行（报 crash-case: 行原文与合表行），不写登记总条数（原话「7a. 新写的崩溃枚举用例（」） |
| F11 | .claude/kb/checks-owed.en.md:891 | 不相干 | 说的是 C116 或 C492 对应的流登记成崩溃枚举用例与放量分项、哪两行变异靶它，没有写登记总条数或合表行数（原话「/ C116 / Leaf 」） |
| F11 | .claude/kb/checks-owed.en.md:901 | 不相干 | 说的是 C116 或 C492 对应的流登记成崩溃枚举用例与放量分项、哪两行变异靶它，没有写登记总条数或合表行数（原话「/ C492 / Writi」） |
| F11 | .claude/kb/checks-owed.ja.md:891 | 不相干 | 说的是 C116 或 C492 对应的流登记成崩溃枚举用例与放量分项、哪两行变异靶它，没有写登记总条数或合表行数（原话「/ C116 / リーフコン」） |
| F11 | .claude/kb/checks-owed.ja.md:901 | 不相干 | 说的是 C116 或 C492 对应的流登记成崩溃枚举用例与放量分项、哪两行变异靶它，没有写登记总条数或合表行数（原话「/ C492 / ツリーテー」） |
| F11 | .claude/kb/checks-owed.md:348 | 要改 | 门禁 checker-tier-crash-point-replay 的 `--full` 已登记的崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行，条数用 `grep -c '^crash-case:'` 现数，正文不抄数）；34 这个数结束提交那一版已是 51，基准那一版就是 47，早就过时（原话「已登记 34 条崩溃枚举用例」） |
| F11 | .claude/kb/checks-owed.md:589 | 要改 | 门禁 `checker-tier-crash-point-replay` 的 crash-case-replay 格对每条没带 `red-blocks=judged-by-test` 的登记用例（条数现数、正文不抄数；结束提交那一版登记 51 条、带该标记 2 条，故查库 49 条，推的）逐条带 `SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1` 起、只查库……（36 是 2026-10-24 那一趟「登记 38、实跑 2、查库 36」的数）；英日同一行同改，见 M4、M5（原话「36 条登记用例」） |
| F11 | .claude/kb/checks-owed.md:891 | 不相干 | 说的是 C116 或 C492 对应的流登记成崩溃枚举用例与放量分项、哪两行变异靶它，没有写登记总条数或合表行数（原话「crash-point-re」） |
| F11 | .claude/kb/checks-owed.md:901 | 不相干 | 说的是 C116 或 C492 对应的流登记成崩溃枚举用例与放量分项、哪两行变异靶它，没有写登记总条数或合表行数（原话「/ C492 / 树表 0 」） |
| F11 | .claude/kb/milestone/04-fourth-txn.en.md:25 | 不相干 | 说的是里程碑四出口 10 的积压登记表按合表项序与放量分项序、每项态数现数，逐项登记；没写合表行数或登记条数。全域约 3.8×10¹⁰ 态是该出口写的前面没跑的全量，新增四条属「本里程碑新增」，不受这件事实影响（原话「/ 10 / Crash f」） |
| F11 | .claude/kb/milestone/04-fourth-txn.ja.md:25 | 不相干 | 说的是里程碑四出口 10 的积压登记表按合表项序与放量分项序、每项态数现数，逐项登记；没写合表行数或登记条数。全域约 3.8×10¹⁰ 态是该出口写的前面没跑的全量，新增四条属「本里程碑新增」，不受这件事实影响（原话「/ 10 / クラッシュフル」） |
| F11 | .claude/kb/milestone/04-fourth-txn.md:25 | 不相干 | 说的是里程碑四出口 10 的积压登记表按合表项序与放量分项序、每项态数现数，逐项登记；没写合表行数或登记条数。全域约 3.8×10¹⁰ 态是该出口写的前面没跑的全量，新增四条属「本里程碑新增」，不受这件事实影响（原话「crash-point-re」） |
| F11 | .claude/kb/milestone/05-fifth-txn.en.md:66 | 不相干 | 说的是该作业点要把新流登记进 stage-inputs.tsv 与合表（待办的登记动作），没写登记总条数，不受 47 到 51 影响（原话「crash-point-re」） |
| F11 | .claude/kb/milestone/05-fifth-txn.en.md:119 | 不相干 | 说的是该作业点要把新流登记进 stage-inputs.tsv 与合表（待办的登记动作），没写登记总条数，不受 47 到 51 影响（原话「crash-point-re」） |
| F11 | .claude/kb/milestone/05-fifth-txn.ja.md:66 | 不相干 | 说的是该作业点要把新流登记进 stage-inputs.tsv 与合表（待办的登记动作），没写登记总条数，不受 47 到 51 影响（原话「crash-point-re」） |
| F11 | .claude/kb/milestone/05-fifth-txn.ja.md:119 | 不相干 | 说的是该作业点要把新流登记进 stage-inputs.tsv 与合表（待办的登记动作），没写登记总条数，不受 47 到 51 影响（原话「crash-point-re」） |
| F11 | .claude/kb/milestone/05-fifth-txn.md:66 | 不相干 | 说的是该作业点要把新流登记进 stage-inputs.tsv 与合表（待办的登记动作），没写登记总条数，不受 47 到 51 影响（原话「crash-point-re」） |
| F11 | .claude/kb/milestone/05-fifth-txn.md:119 | 不相干 | 说的是该作业点要把新流登记进 stage-inputs.tsv 与合表（待办的登记动作），没写登记总条数，不受 47 到 51 影响（原话「crash-point-re」） |
| F11 | .claude/kb/milestone/11-eleventh-txn.en.md:66 | 不相干 | 说的是该作业点要把新流登记进 stage-inputs.tsv 与合表（待办的登记动作），没写登记总条数，不受 47 到 51 影响（原话「crash-point-re」） |
| F11 | .claude/kb/milestone/11-eleventh-txn.ja.md:66 | 不相干 | 说的是该作业点要把新流登记进 stage-inputs.tsv 与合表（待办的登记动作），没写登记总条数，不受 47 到 51 影响（原话「crash-point-re」） |
| F11 | .claude/kb/milestone/11-eleventh-txn.md:67 | 不相干 | 说的是该作业点要把新流登记进 stage-inputs.tsv 与合表（待办的登记动作），没写登记总条数，不受 47 到 51 影响（原话「crash-point-re」） |
| F11 | .claude/rules/verification.md:13 | 不相干 | 说的是 crate 模块清单里积压登记表的读写、表在 crate 根，没写表的行数（原话「crash-full-run」） |
| F11 | .claude/rules/verification.md:75 | 不相干 | 说的是登记一条全量用例要改哪几处（stage-inputs 一行、合表一项、placement 检查），是流程规则，不写条数（原话「crash-point-re」） |
| F11 | .claude/rules/verification.md:129 | 不相干 | 规则表：崩溃枚举用例登记进合表后门禁开跑前跑 placement,modules 检查，不写条数（原话「crash-point-re」） |
| F11 | .claude/skills/crash-test/SKILL.md:34 | 不相干 | 说的是登记写法（stage-inputs 一行 crash-case:<名>、合表一项在第二列点名），不写条数（原话「crash-point-re」） |
| F11 | README.md:84 | 事件句不改 | README 的历史全量记录，说的是 2026-10-16 到 2026-10-18 那一轮全量罩 14 条，已标「历史……不代表当前工作区」（原话「14 条崩溃枚举用例，日志打」） |
| F11 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:70 | 事件句不改 | 2026-09-28 记录里当时 8 条崩溃枚举用例各收 41 到 46 份文件，是那一次闭包探针的结果（原话「8 条崩溃枚举用例各收 41」） |
| F11 | records/2026-09-30-里程碑三开工第一轮.md:35 | 事件句不改 | 2026-09-30 当时 8 条登记用例都不经 CrashPointRecorder 的核对，是那一次的核对（原话「8 条登记用例」） |
| F11 | records/2026-09-30-里程碑三开工第一轮.md:973 | 事件句不改 | 2026-09-30 当时的合表 13 项与轻型登记表 254 行，是那一次的核对结果（原话「crash-point-re」） |
| F11 | records/2026-09-30-里程碑三开工第一轮.md:999 | 事件句不改 | 当时 6 条崩溃枚举用例闭包的探针结果，是那一次的事（原话「6 条崩溃枚举用例闭包不再含」） |
| F11 | records/2026-10-11-scripts审计.md:51 | 事件句不改 | 2026-10-11 脚本审计的发现（合表里 7 项同时有第二列与第三列），是当时的核对（原话「crash-point-re」） |
| F11 | records/2026-10-11-崩溃放量两段与脚本审计.md:77 | 事件句不改 | 2026-10-12 那趟只跑了 12 条的经过（登记表里当时共 13 条），是那一趟的事（原话「12 条崩溃枚举用例（登记表」） |
| F11 | records/2026-10-11-里程碑三知识腐烂治理.md:30 | 事件句不改 | 2026-10-11 知识腐烂治理当时的现数（crash-case: 13 条、合表 13 项），是那一次盘点的结果（原话「crash-point-re」） |
| F11 | records/2026-10-11-里程碑三知识腐烂治理.md:32 | 事件句不改 | 回扫员当时报的「C564 写已登记 9 条（现数 13）」，是那一次盘点的发现（原话「已登记 9 条崩溃枚举用例」」） |
| F11 | records/2026-10-13-代码欠账批.md:186 | 事件句不改 | 2026-10-13 当时全量 14 条已判绿 5 条、第 6 条在跑，是那一趟的进度（原话「14 条崩溃枚举用例已判绿 」） |
| F11 | records/2026-10-13-复用键与全仓改名.md:98 | 事件句不改 | 当时加登记行并核「合表 14 项」的操作，是那一次的事（原话「crash-point-re」） |
| F11 | records/2026-10-13-复用键与全仓改名.md:846 | 事件句不改 | 当时关于 C789 未立的闸与观察者罩整条流状态数的说明，是那一次的核对（原话「crash-point-re」） |
| F11 | records/2026-10-13-复用键与全仓改名.md:1050 | 事件句不改 | 当时一趟全量的日志结果（1 条用例这一趟都判绿），是那一趟的事（原话「1 条崩溃枚举用例这一趟都判」） |
| F11 | records/2026-10-13-里程碑四预开发.md:63 | 事件句不改 | 当时别的会话改动清单里的描述，是那一次的事（原话「- 用户 2026-10-1」） |
| F11 | records/2026-10-13-里程碑四预开发.md:168 | 事件句不改 | 当时主 agent 照报告原文登三行的操作，是那一次的事（原话「crash-point-re」） |
| F11 | records/2026-10-13-里程碑四预开发.md:185 | 事件句不改 | 当时主 agent 照报告原文登四条流的操作，是那一次的事（原话「- 崩溃流第四波交回（Opu」） |
| F11 | records/2026-10-13-里程碑四预开发.md:312 | 事件句不改 | 当时插入三处登记行并重核的操作，是那一次的事（原话「- 截戊交回（Opus，12」） |
| F11 | records/2026-10-13-里程碑四预开发.md:703 | 事件句不改 | 当时补登记（stage-inputs 5 行、合表 4 行等）的操作，是那一次的事（原话「crash-point-re」） |
| F11 | records/2026-10-13-里程碑四预开发.md:1138 | 事件句不改 | 当时补轻型登记表与各登记行的操作，是那一次的事（原话「crash-point-re」） |
| F11 | records/2026-10-13-里程碑四预开发.md:1150 | 事件句不改 | 工具员交回出口 10 的入口与 crash-full-run-backlog 格，是那一次交回的事（原话「crash-full-run」） |
| F11 | records/2026-10-13-里程碑四预开发.md:1151 | 事件句不改 | 2026-10-26 主 agent 补 crash-full-run-backlog 一行等的操作，是那一次的事（原话「crash-full-run」） |
| F11 | records/2026-10-17-全仓知识腐烂治理.md:64 | 事件句不改 | 当时改登记表头注（合表 13 到 14 条）的操作，是那一次的事（原话「- 批 5 主 agent 」） |
| F11 | records/2026-10-18-审查报告核实与修复.md:53 | 事件句不改 | 当时把合表文件头的「14 条与 8 项」改成「全部」的操作，是那一次的事（原话「crash-point-re」） |
| F11 | records/2026-10-18-审查报告核实与修复.md:120 | 事件句不改 | 当时派工具员并与别的会话协调往两张表 insert-row 的经过，是那一次的事（原话「crash-point-re」） |
| F11 | records/2026-10-18-里程碑三收尾与近期欠账.md:50 | 事件句不改 | 当时不碰这些文件并约定以后用 insert-row 追加登记行，是那一次的约定（原话「crash-point-re」） |
| F11 | records/2026-10-18-里程碑三收尾与近期欠账.md:86 | 事件句不改 | 当时登记三处的操作（现 25 行是当时的数），是那一次的事（原话「crash-point-re」） |
| F11 | records/2026-10-18-里程碑三收尾与近期欠账.md:89 | 事件句不改 | 当时 diff 打进两张登记表的操作，是那一次的事（原话「crash-point-re」） |
| F11 | records/2026-10-18-里程碑三收尾与近期欠账.md:773 | 事件句不改 | 当时 mkfs 用例两行登记的操作，是那一次的事（原话「crash-point-re」） |
| F11 | records/2026-10-18-里程碑三收尾与近期欠账.md:1199 | 事件句不改 | 当时追加登记行并备份的操作，是那一次的事（原话「crash-point-re」） |
| F11 | records/2026-10-18-里程碑三收尾与近期欠账.md:1478 | 事件句不改 | 当时基线红 3 行的现象，是那一趟的事（原话「crash-point-re」） |
| F11 | records/2026-10-18-里程碑三收尾与近期欠账.md:1482 | 事件句不改 | 当时基线红去向的处理，是那一次的事（原话「crash-point-re」） |
| F11 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:490 | 事件句不改 | 2026-10-19 当时快档两格报 33 条本次未跑，是那一趟的结果（原话「33 条崩溃枚举用例全量本次」） |
| F11 | records/2026-10-21-doc门禁违规修复.md:97 | 事件句不改 | 当时列出别的会话要改的文件清单，是那一次的事（原话「crash-point-re」） |
| F11 | records/2026-10-21-doc门禁违规修复.md:143 | 事件句不改 | 当时写回员写「已登记 35 条」、主 agent 照 HEAD 改成 34 的经过，是那一次的事（原话「已登记 35 条」，主 ag」） |
| F11 | records/2026-10-24-checker审计复核修复.md:101 | 事件句不改 | 2026-10-24 那趟快档日志「登记 38 条、查库 36 条」，是那一趟的结果（原话「38 条崩溃枚举用例：实跑 」） |
| F11 | records/2026-10-24-checker审计复核修复.md:102 | 事件句不改 | 同一趟的监控结果（本次未跑 36 条），是那一趟的结果（原话「36 条崩溃枚举用例的全量（」） |
| F11 | records/2026-10-25-全量提交门禁实跑与崩溃重放状态复用报告.md:5 | 事件句不改 | 2026-10-25 那趟全量提交门禁实跑报告，写的是当时登记 38 条与执行进度，是那一趟的事（原话「38 条崩溃枚举用例逐条在独」） |
| F11 | records/2026-10-25-全量提交门禁实跑与崩溃重放状态复用报告.md:28 | 事件句不改 | 2026-10-25 那趟全量提交门禁实跑报告，写的是当时登记 38 条与执行进度，是那一趟的事（原话「38 条崩溃枚举用例中，前 」） |
| F11 | records/2026-10-25-全量提交门禁实跑与崩溃重放状态复用报告.md:50 | 事件句不改 | 2026-10-25 那趟全量提交门禁实跑报告，写的是当时登记 38 条与执行进度，是那一趟的事（原话「31 条崩溃枚举用例」） |
| F11 | records/2026-10-26-checker欠账C896-C897-C914.md:102 | 事件句不改 | 2026-10-26 第二台那趟「登记 45 条、查库 43 条」，是那一趟的结果（原话「45 条崩溃枚举用例：实跑 」） |
| F11 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:54 | 事件句不改 | 这一批调度记录：四条新用例在暂存区新登记、全量一次没跑过，是当时的核对（原话「crash-point-re」） |
| F11 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:103 | 事件句不改 | 用户 2026-10-28 追加要求与验收标准，是那一次的事（原话「crash-point-re」） |
| F11 | records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md:211 | 事件句不改 | 批次 交回里主 agent 改登记注释六处的操作，是那一次的事（原话「crash-point-re」） |
| M1 | .claude/kb/milestone/04-fourth-txn.md:25 | 要补 | 这一阶段做成的四条新崩溃枚举用例与假续跑根因修复，04-fourth-txn.md 没有一处记着（结束提交那一版该文件没改，grep 假续跑、compaction-deadlist、three-generation、tree-split-and-shrink、last-fitting 全 0 命中）。出口 10 要求「新增的跑完」，这几条正是本里程碑新增、全量没判过的流。补在出口 10 后或「3 覆盖」行：「2026-10-28 起新增四条崩溃枚举用例（compaction-deadlist-reclamation、three-generation-mount-fork、tree-split-and-shrink、last-fitting-unit-then-refusal-and-unmount-on-a-nearly-full-pool）已登记 stage-inputs.tsv 的 crash-case: 行、合表与积压登记表（crash-full-run-backlog.tsv，状态数照闭式计数填、全量没判过）；2026-10-27 双机退 101 的四条老用例，根因（键变后切片假续跑）已修，这八条的全量重跑用户 2026-10-29 定这一轮不起」，出处 records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md；英日同补。若主 agent 认为该文件不记这类进度，改判不相干 |
| M2 | .claude/kb/checks-owed.en.md:348 | 要改 | 候选表没罩到（检索词只有中文）：C564 英文行写「has 34 crash enumeration cases registered (the `crash-case:` rows … currently counted by `grep -c '^crash-case:'`)」，34 已过时（结束提交那一版 51）；改成「has the crash enumeration cases registered (the `crash-case:` rows of `.claude/gate.d/stage-inputs.tsv`, count them with `grep -c '^crash-case:'`; the text does not copy the number)」，与中文行 261 同改 |
| M3 | .claude/kb/checks-owed.ja.md:348 | 要改 | 候選表の検索語が中国語のみで拾えていない同源の行：C564 日本語行「`--full` には 34 本のクラッシュ列挙のテストケースが登録済み」の 34 は古い（結束コミット版は 51）；「登録済みのクラッシュ列挙ケース（本数は `grep -c '^crash-case:'` で数え、本文には書き写さない）」に直す。中国語行 261 と同改 |
| M4 | .claude/kb/checks-owed.en.md:589 | 要改 | 候选表没罩到：C914 英文行写「starts each of the 36 registered cases with `SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1` to query the library only」，36 是 2026-10-24 那趟的数；改成「each registered case not marked `red-blocks=judged-by-test` (count them, the text does not copy the number)」，与中文行 262 同改 |
| M5 | .claude/kb/checks-owed.ja.md:589 | 要改 | 候補表で拾えていない同源の行：C914 日本語行「登録された 36 ケースをそれぞれ `SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1` 付きで起動」の 36 は 2026-10-24 の数；「`red-blocks=judged-by-test` を付けていない登録ケースをそれぞれ（本数は数えて、本文には書き写さない）」に直す。中国語行 262 と同改 |
| M6 | .claude/kb/milestone/05-fifth-txn.en.md:153 | 要改 | 候选表没罩到（检索词只有中文）：VIII·3 英文行同样列着「C786 (Importing multiple library copies per plan counts same block once per copy)」与做什么列的「blocks imported across two-host counted only once」；C786 已判还清，同中文行 180 去掉这一项 |
| M7 | .claude/kb/milestone/05-fifth-txn.ja.md:153 | 要改 | 候補表で拾えていない同源の行：八·3 日本語行の「C786 (計画に従い複数ライブラリをインポートする際に同一ブロックを各コピーでカウント)」と、做什么列の二台インポートに関する目標。C786 は解決済みなので、中国語行 180 と同じく外す |

## 反向问：这一阶段做成的事，在该记它的载体里有没有一处记着（第 10 步）

核法：对结束提交那一版（`git show 结束提交:路径 | grep -n …`）按下面各事的关键词在各载体里搜；不搜 `research/prompts/` 与 `briefs/` 的回改。

| 做成的事 | 调度记录 | `.claude/kb/checks-owed.md` | 拆分提案 | `.claude/kb/milestone/04-fourth-txn.md` | 结论 |
|---|---|---|---|---|---|
| 假续跑根因修了（导入不早退、观察者红计入、块查询连观察者结论问） | 有（批次 4、5、7 各节） | 不需要新条（C786 判还清已在工作区调度记录「验证段收尾」） | 不需要 | 没有（关键词 假续跑 0 命中） | 04 缺，见 M1 |
| `imported_blocks=` 同为块数、GPU 判器组按路线取、`admission.py` 零块不算罩住 | 有（D5、D6、D7 与 8b 各节） | C786 三语在结束提交里仍开着，见 177-179 | 不需要 | 不需要 | 已处理，要改 3 行 |
| 四条新用例写实并登记 | 有（第 54、211 行等） | 不需要 | 不需要 | 没有（compaction-deadlist、three-generation、tree-split-and-shrink、last-fitting 全 0 命中） | 04 缺，见 M1 |
| 55 行变异逐条证红 | 有（第 278 行） | 不需要 | 不需要 | 不需要 | 已记 |
| `multi-host-run.sh --peer` 拒带判定库的命令；`detached-run.sh watch` 任何单元结束当场报并退 | 有（批次 11 各节） | 不需要 | 有（一百四十八节） | 不需要 | 已记 |
| 新立 C924、C925 与拆分提案三节 | 有 | 有（`checks-owed.md` 第 597、598 行，三语各加 2 行） | 有（一百四十八、一百四十九、一百五十节） | 不需要 | 已记 |
| 全量重跑这一阶段没起 | 有（第 312 行，用户 2026-10-29「1 不要 2 要 3 要」） | 不需要 | 不需要 | 没有 | 04 缺，并进 M1 |

## 没做什么

- 没改任何文件；M1-M7 只给改法，不改。
- 候选表只罩事实表写到的事实、检索词搜得到的句子：C564、C914 与 05-fifth-txn 八·3 的英日行因检索词只有中文没进候选，是我顺着同一条目反向 `git grep` 找到的（命令：`git grep -n 'C786' 结束提交 -- '*.md' ':!records' ':!research'`，以及对 `34`、`36` 的登记条数的 `git grep -nE`）；别的语种、别的写法里还有没有同类句我只扫了 `*.md`、`*.sh`、`*.py`、`*.tsv`（排除 records、research、briefs），`crates/` 里的注释与钉值不在本份候选内，没逐条判。
- 没读主工作区以外的会话记录；「用户 2026-10-29 定不起」「C786 判还清」都取自调度记录与主工作区文件，未另核。
- 判事件句时依据是行的写法与所在小节的日期；若主 agent 想让某条过程记录里的待办句也随这一阶段改写，请指出，我这里一律按「不回改过程记录」判。
- 没跑任何测试、没起任何编译。
- 推翻条件：若 `git show 结束提交:.claude/gate.d/stage-inputs.tsv | grep -c '^crash-case:'` 不是 51，或主工作区 `checks-owed.md` 第 933 行不是 C786，则 262、261、177-179 的改后句子要重写。

## 命令输出

- 「交实现员」数行：`grep -E '^\| (F3|F4|F5|F11|M[0-9]+) \| crates/' <本报告> | grep -cE '\| (要改|要补) \|'` 输出 `0`。
- 逐行判定行数：`grep -c '^| \(F3\|F4\|F5\|F11\) |' <本报告>` 输出 `147`，与候选表这四组行数相等；按判定数：要改 6（F3 4、F11 2）、事件句不改 104（17+34+10+43）、不相干 37（5+9+23）。
- `python3 scripts/kb/stale-candidates.py --check-report 候选表 本报告 --groups F3-F5,F11` 退 0，输出末行：
  `✓ 报告判全了：147 行候选都有逐行判定；crates/ 下判要改要补的 0 行都在「交实现员」一节里（不变量清单的 .md 除外）`
