# 子 agent 落地审计：调查员 18 个、回扫员 14 个（2026-10-22）

批次：`records/2026-10-22-子agent结果落地审计.md#3`。只读：仓内没改文件，没做 git 写操作，没编译，没跑测试。
输入：`/home/fy5090/.local/share/singlefs/work/subagent-landing-audit-2026-10-22/inputs/3-investigator-sweep.json`（序号 0–17 是调查员，18–31 是回扫员，下文「#n」就是这个序号）。

## 结论

- 调查员 18 个：修复落了地 13、结论记进了文档 5、结论被否了 0、没下文 0、判不了 0。
- 回扫员 14 个：有「要改 / 要补」的 12 个，合计建议改 502 行，已改 479 行（95.4%）；被别的改动取代 9 行，主 agent 判了不改 2 行，漏了 12 行。
  #28 一个就占 334 行；不算它，建议改 168 行、已改 147 行（87.5%）。另两个（#22、#25）做的是事实表，报告里没有要改要补的行。
- 漏掉的 12 行全在 #29 一份报告里。没下文的另一块是 #10 调查员给的两条改法，主 agent 记成「延后」，后面没人接。共同原因见「汇总」一节。
- 推翻条件：表里标「已改」的行，如果它的旧串在仓里还找得到、或新串不在，就要改判；标「漏了」的行，如果在别的文件里找到了同义的改写，也要改判。逐行核的命令与原样输出在 `evidence-sweeps.txt`、`evidence-investigators.txt`（草稿目录，sha256 见文末）。

## 查法

- 交回原文取自输入 JSON 的 `handback`。报告文件在的（#4、#6、#8、#15）没重读全文，只用交回里的结论。报告文件已不在的回扫员，用 `tools/sub.py <序号> writes` 从子 agent 会话记录里抽出写报告的那几步（Write、`cat >`、生成脚本里的判定行），落在 `rep/<序号>.writes.txt`。
- 主会话的后续动作用 `tools/main.py <序号> <截断> <正则>` 取：只看子 agent 结束时刻之后的条目（没有时间戳的条目沿用上一条的时间），看有没有把报告派给实现员、写回员，或记进调度记录。
- 仓里现状：调查员这一边逐条 `grep -n` 现取行号（`evidence-investigators.txt`，83 行）；回扫员这一边把每行的旧串或新串写进 `sweep-rows.tsv`，用 `tools/sweep-check.sh` 逐行 `git grep -c -F`（`evidence-sweeps.txt`，95 行）。旧串不在算「已改」，新串在算「已改」。
- `.claude/kb/checks-owed.md` 在核的同一天被别的会话改着（文件修改时间在本审计当天），行号以 `evidence-investigators.txt` 那一次为准，引用时同时写欠账号。

## 表一：调查员逐条

| # | 描述（交回日期） | 判定 | 依据（文件:行现取，或提交号） |
|---|---|---|---|
| 0 | 慢档两条红：分来源（10-03） | 修复落了地 | 问 1（吞写用例取样点随比重表丢了）派给第 12 批实现员 Z（派发提示点名本报告问 1），用例改成写死历史加一个注入点：`crates/singlefs-harness/tests/fault_injection_fast_tier.rs:581` 文档注释写明来历，`git log -S '慢档红调查问 1'` 首次出现在 6c1509edd（2026-10-03）。问 2（5 条新发现是 C600 那一形）记进 `records/2026-10-01-删除与并发事务.md:391`，归 R2 那条线 |
| 1 | 核证欠账 庚组（10-11） | 结论记进了文档 | 主 agent 照拟写句子把 6 笔挪进已还清表：`checks-owed.md:765–769`（C435、C634、C642、C649、C696，末列 2026-10-11），C673 在 `:785`；半还 4 笔改成只剩还欠那一半：C521 `:334`、C702 `:439`、C770 `:498`；C450 之后真修了，`:813` 末列 2026-10-13。记录 `records/2026-10-11-里程碑三知识腐烂治理.md:149` |
| 2 | 扫描内核展开超预算的来源（10-06） | 修复落了地 | 派第 10q 批实现员照它的补丁并调用点，`records/2026-09-30-里程碑三开工第一轮.md:741`（交回）、`:755`（合入）；现状 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:4057` 只剩一处 `d_judge_deletion_state_of_a_version(` 调用 |
| 3 | 核证欠账 丁组（10-11） | 结论记进了文档 | 已还 C536、C670、C704、C710 与作废 C705 在已还清表 `checks-owed.md:741–745`（末列 2026-10-11）；半还 C188 `:156`、C406 `:285`、C645 `:406` 改成只剩还欠那一半。分组表 `records/2026-10-11-里程碑三知识腐烂治理.md:124` |
| 4 | 出口 4 摸底：覆盖表与评估次数（10-19） | 修复落了地 | 主 agent 照报告第五节弹窗定 K1–K7，按它建议的簇派实现员；门禁格已在 `.claude/gate.d/checker-tier-crash-point-replay.sh:11`、`:13`（crash-flow-coverage、invariant-zero-evaluations），随 767668191 提交。记录 `records/2026-10-18-里程碑三收尾与近期欠账.md:317`、`:329`、`:340` |
| 5 | 复用键审计路子三（日志）（10-14） | 修复落了地 | 与 #7 一起并成对账表 `records/2026-10-13-复用键与全仓改名.md:438`（28 条，用户定全部改）；之后按函数单元键重做，提交 df5866f67（2026-10-18「判定按函数单元键复用」）；它点名的 `--changed-since` 已删，`scripts/mutation/broad-mutation.py:28` |
| 6 | T5 标签 0 改 size 的 panic（10-22） | 修复落了地（还没提交） | 弹窗定「丑：写者接得住这一形」，当天派实现员：`records/2026-10-13-里程碑四预开发.md:890`（交回）、`:908`（实现员交回）；`crates/singlefs-core/src/truncate.rs:241` 断言已放宽，暂存区 `1 file changed, 10 insertions(+), 11 deletions(-)` |
| 7 | 复用键审计路子二（实现）（10-14） | 修复落了地 | 同 #5：对账表「哪几路」一列只写「二」的第 2、3、8、9、11、19、20 条只有它报，其余多条与另两路重合；df5866f67；它排第一的「--changed-since 下一律现判」已删（`broad-mutation.py:28`） |
| 8 | C659 mkfs 与 C699、C395（10-20） | 修复落了地 | 结论进了 C656 第二、三轮，用户 2026-10-20 定「强制新 fsid」（`records/2026-10-18-里程碑三收尾与近期欠账.md:376`、`:418`）；`crates/singlefs-core/src/make_filesystem.rs:9` 对外入口从随机源取 fsid；C659 改简称、钉坏镜像，`checks-owed.md:852` |
| 9 | 核证欠账 甲组（10-11） | 结论记进了文档 | 已还 C46、C415、C429、C643 在 `checks-owed.md:729–732`（2026-10-11）；它判半还、未还的 6 笔后来都还了：C646 `:802`、C455 `:810`、C531 `:814`（2026-10-13），C589、C590、C612 `:867–869`（2026-10-22） |
| 10 | 快照创建分项读不到录入段清单（10-20） | 结论记进了文档 | 根因记进 `records/2026-10-13-里程碑四预开发.md:752`；它给的改法记成「延后」（`:753` 入口钉源码戳、`:754` 行与报错带录入键），没有接手：`crash_amplification.rs:13665` 的 `CRASH_AMPLIFICATION_RECORDED` 行仍不带录入键，`checks-owed.md` 里 `整条 use`、`录入键` 都是 0 处 |
| 11 | 28-1 没判计数 1 变 4（10-05） | 修复落了地 | 主 agent 当天跑了它的五副本命令；改用例：`crates/singlefs-checker-tier/src/crash_injection.rs:8277` `Some(1 + 3)`，它建议的变异行在 `crates/mutations.tsv:2527`，两样都随 f8a9ddaab（2026-10-10）提交 |
| 12 | 核证欠账 乙组（10-11） | 结论记进了文档 | 已还 C626、C627、C601、C607、C608、C620、C674、C676 在 `checks-owed.md:733–740`（2026-10-11）；半还 C604 `:379`、C675 `:422` 照它的说法只留还欠那一半 |
| 13 | 清场包装回读失败（10-06） | 修复落了地 | 用户授权后主 agent 改私有清场脚本，包装一半派工具实现员照它的 diff 做：`scripts/crash/gpu-quiesce-run.sh:49`（`stop-exit=`、`log=`），记录 `records/2026-09-16-subagent拆分提案.md:1152` |
| 14 | HEAD 上 9 条 checker 档红归属（10-04） | 修复落了地 | 主 agent 照它定 #1 补登记，派 26-2 改 #1–4、#6–#9：`records/2026-10-01-里程碑三形式化接入.md:955`（交回）、`:981`（26-2 交回）、`:1004`（合入验证） |
| 15 | 变异轻型的红与基线没判成（10-18） | 修复落了地 | 问 5 的 PicklingError 照它的根因改：`scripts/mutation/lib_mutation_units.py:126` 登记进 `sys.modules`；记录 `records/2026-10-18-里程碑三收尾与近期欠账.md:189` |
| 16 | E171 流三 I-3.1 红（10-05） | 修复落了地 | 装置装上根环表（它的改法 a）：`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:1161`；改法 b 的 C551 已还清 `checks-owed.md:725`（2026-10-06）；记录 `records/2026-10-01-里程碑三形式化接入.md:1079` |
| 17 | 六种测试核查第 1 路（10-18） | 修复落了地 | 三路并成 11 条后主 agent 逐条现查（`records/2026-10-13-里程碑四预开发.md:640`）；它的 M1（wgsl 变异键取整份文件）已改成按 fn 切 `scripts/mutation/lib_mutation_units.py:311`，S1（`line:enable-gpu` 进键）已去掉 `.claude/gate.d/checker-tier-crash-point-replay.sh:366` |

## 表二：回扫员逐个

「建议改」是报告里判「要改」「要补」的行数（#20、#27 的口径见备注）；「已改」按 `evidence-sweeps.txt` 与下面写的现查；「没改」再分「取代」（被别的改动改写，原句不在了或重新成真）、「主 agent 不改」、「漏」。

| # | 描述（交回日期） | 建议改 | 已改 | 没改（取代 / 主 agent 不改 / 漏） | 怎么落的、备注 |
|---|---|---|---|---|---|
| 18 | 第二轮通读 R4：skill 与里程碑（10-17） | 10（改 9、补 1） | 10 | 0 | 主 agent 自己落：`records/2026-10-17-全仓知识腐烂治理.md:77`「R4 的 9 处要改与 1 处要补主 agent 自己落」。commit-stages:19、:32 没照它的拟句，改成按阶段脚本文件头取输入（它自己也说路径集是暂拟） |
| 19 | 第二轮 Q1：路径·不变量·带日期现状（10-11） | 8（改 8） | 8 | 0 | 第二轮写回（`records/2026-10-11-里程碑三知识腐烂治理.md:89`「第二轮写回收齐」）。里程碑二的三行新句与拟句逐字一致；里程碑五那一行随 10-13 里程碑重排挪到今天的 `milestone/04-fourth-txn.md`，新句在 |
| 20 | 欠账全表分诊第 3 段（10-11） | 20（疑似已还 3、重复 5、半还 12） | 20 | 0 | 分诊报告没有「要改 / 要补」两类，按它要动登记行的三类算。重复 5 笔并进已还清表 `checks-owed.md` C431、C449、C477、C529、C534 各一行（2026-10-11）；半还 12 笔都改成「已有 / 仍欠」两截；疑似已还 3 笔：C435 已还清，C450 修了之后 2026-10-13 已还清，C521 经 #1 核证改判半还、按半还改写 |
| 21 | 逐行判 S33（10-11） | 13（改 13） | 13 | 0 | 第一轮写回；C692 挪进已还清表（2026-10-11） |
| 22 | 事实表丙丙：里程碑 03–05（10-17） | 0 | — | — | 事实表，不判行。它交主 agent 现查的 6 条里 4 条今天已对上（03 待优化「写者锁那一半没做」0 处、两份里程碑「文末历史版本记一行」0 处、`walk.rs:10247` 的 C314 简称与欠账表一致、E179 页在）；带日期现查快照与 C58 去向两条没逐一核 |
| 23 | 逐行判 S29（10-11） | 15（改 15） | 15 | 0 | 输入里桶是 report-unreferenced，实际全落：欠账 7 行、`layout/01` 1 行、`milestone/02` 5 行、`records/2026-09-16-subagent拆分提案.md:958` 与 `:1034` |
| 24 | 逐行判 S22（10-17） | 10（改 10） | 10 | 0 | C609、C671、C639、C722 两处、`milestone/03` C650 与行 38 都改了；行 38 后来又被改写成「这 9 行」 |
| 25 | 事实表丙：决策与实验正文差异（10-11） | 0 | — | — | 事实表，不判行，产物进了候选表 |
| 26 | 逐行判 S23（10-11） | 25（改 24、补 1） | 25 | 0 | C675 那一行它判「已还清」，主 agent 照 #12 核证改写成半还（CPU 一半已有、GPU 一半仍欠，`checks-owed.md:422`），算已改但没照它的判定 |
| 27 | 乙：五·5 各行与落后的两行（10-11） | 21（补 17、改 2、E150 两处） | 21 | 0 | 17 行前置末尾都补了「去向：…」，主题后来随 10-13 里程碑重排换成里程碑号；C600 `:376`、C368 改成「已做 / 仍欠」；E150 标题与 `experiments.md` 索引行写明「结论只作历史」 |
| 28 | 扫 k1 指代与时间（10-11） | 334（改 334） | 332 | 2（主 agent 不改） | 主 agent 判「（日期 现查）」的日期是出处、要留，9 处退回，其中 2 项退回后与原文相同、不写；`replace-batch` 写回 332 处、回读一致（主会话那一步原样输出 `334 332 restored 9 dropped 2` 与 `✓ 332 处替换，写了 1 个文件（改名换上新 inode），回读一致`）；`records/2026-10-11-指令文档删论证与来历.md:54` |
| 29 | E 批之后的知识腐烂回扫（10-11） | 20（现状腐烂 20） | 7 | 13（取代 1、漏 12） | 见下面逐行 |
| 30 | 逐行判 S35（10-17） | 14（改 14） | 8 | 6（取代 6） | 简报引用 6 行：简报三语后来改成 `.md` 中文、`.en.md` 英文，`briefs/2026-09-28.md` 里「全量（2026-09-30）」现在 1 处，原引用重新成真，不用改。其余 8 行（tooling 格范围、`cd research` 6 处、verification-build 八个 crate 与条数）已改。`evidence-sweeps.txt` 里「cd research 6 处」那一行报 7 处，是范围放宽到整个 experiments 目录后命中了 116、158、171 三页的 `cd research/results`（不是它点的行）；点名的 173、176、177、178、95 与 `vm-harness.md` 逐份数，`cd research` 0 处、`cd scripts/experiment-apparatus` 各 1 处（95 页 0 处，它要改的是一行注释） |
| 31 | 第二轮 A3：通读里程碑四五与布局（10-11） | 12（改 12） | 10 | 2（取代 2） | 桶是 report-unreferenced，实际落了 10 行。旧里程碑五今天是 `milestone/04-fourth-txn.md`：「第四份」两处、克隆一句、`1 678 377 089` 都照它改了；「里程碑三的并行提交」与段 0 补「里程碑四定下的并行提交形态」两行随 10-13 重排不再成立（`milestone/04-fourth-txn.md:5`：原来的项六（并行提交）搬进里程碑六） |

### 没改的行逐行

#29（报告 `rot-sweep-2026-10-11/report.md` 已不在，行取自它会话里生成主表的 `gen.py`、`gen2.py`；行号是它当时的行号）。主 agent 收它时在 `records/2026-09-30-里程碑三开工第一轮.md:1048` 只把 kb、规则、CLAUDE.md、README 那几行与 `replay.sh:723` 列进「这一阶段的同步一批写回」，`crates/` 里的注释行一行没列。

| 当时位置 | 要改什么 | 现在 | 判 |
|---|---|---|---|
| `.claude/kb/decisions/13-验证路线.md:254` | 「动了判器」路径清单罩不到 `judging_closure.rs`、`build.rs`（它标拿不准） | 这一段已按函数单元键重写（今天第 261 行「按崩溃判定库现算」） | 取代 |
| `.claude/kb/invariants.md:6` | 「GPU 判器只记违例位」改成也记判到位 | 拆分后在 `crates/singlefs-checker/invariants/README.md`，原句还在（命中 1）；`crash_judge_gpu.rs:20` 写着第 16..20 字是判到位，原句今天仍不真 | 漏（列进了写回，没落） |
| `invariants.md:199`、`:200` | 「只有违例位、部分没判不置位」「GPU 判器只有违例位，这几格不置位」 | 在 `invariants/07-分配器.md`，两句都在 | 漏（同上） |
| `invariants.md:290` | 「落判不了（`UNDECIDABLE_TOO_DEEP`），交回 CPU 判」与流水线不合（它标拿不准） | 在 `invariants/09-inode树结构.md:14`，原句在 | 漏（同上） |
| `.claude/rules/verification.md:13` | 模块表漏 `crash_judge_conclusions`、`gpu_card_configuration`，特性半句漏 `crash_amplification` 挂 `verdict-store` | 特性半句已补，两个模块仍没列（`crash_judge_conclusions` 在这份规则里 0 处；`crates/singlefs-checker-tier/src/` 下两份文件都在） | 漏（改了一半） |
| `crates/singlefs-checker-tier/src/crash_judge_gpu.rs:123` | 「结论里只有违例位」后半句不真 | 今天在第 191 行，原句在 | 漏（没派） |
| `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2201`、`:9856`、`:11000`、`:12891` | 「照流水线那样判不了就回落 CPU」与流水线不合 | 这个串今天还有 5 处 | 漏（没派），4 行 |
| `crash_judge_gpu_allocation_record_tables_at_their_capacity.rs:24` | 「多一条的交 CPU」（它标拿不准） | 原文件今天不在，同一句在 `crash_judge_gpu_tables_at_their_capacity.rs:2` | 漏（没派） |
| `crates/singlefs-checker-tier/src/gpu_unit_checks.rs:571` | 「七个分项」改八个 | 今天第 706 行原句在 | 漏（没派） |

已改的 7 行：`CLAUDE.md:8`、`README.md:60`、欠账 C722、`milestone/03:146`、`verification-build.md:283`、`rules/verification.md:35`、`replay.sh:723`（默认冻结根去掉了，今天第 1737 行「根没有默认」）。

#28 没改的 2 行：主 agent 判保留「（日期 现查）」里的日期，退回后与原文相同。判出它们要 `k1/spec.json`，那份文件已不在，具体是哪两行没法再指出来。

#30 取代的 6 行：`milestone/02-second-txn.md:5`、`:484`，`milestone/03-third-txn.md:48`、`:111`、`:139`，`milestone/04-fourth-txn.md:72`（当时行号），都是指简报中文小节的引用。

#31 取代的 2 行：旧 `milestone/05-fifth-txn.md:46`、`:30`（见表二）。

## 汇总

| 调查员判定 | 个数 |
|---|---|
| 修复落了地 | 13（#0、2、4、5、6、7、8、11、13、14、15、16、17；#6 还在暂存区、没提交） |
| 结论记进了文档 | 5（#1、3、9、10、12） |
| 结论被否了 | 0 |
| 没下文 | 0 |
| 判不了 | 0 |

回扫员「要改要补」行落地率：已改 479 / 502 = 95.4%；把取代与主 agent 判了不改也算有了处置，处置率 (479 + 9 + 2) / 502 = 97.6%；不算 #28，已改 147 / 168 = 87.5%。

没下文、漏掉的那部分，共同原因（都是从记录与主会话推的，没有别的证据）：

1. **改动落在 `crates/` 的注释里，写回批不管它。** #29 漏的 12 行里 7 行在 `crates/singlefs-checker-tier`，主 agent 收报告时只把文档行派给写回（`records/2026-09-30-里程碑三开工第一轮.md:1048`），代码注释既没进写回批，也没进实现批。
2. **载体在回扫和写回之间搬了家。** #29 的 4 行不变量句子点的是 `.claude/kb/invariants.md`，同一天它被拆进 `crates/singlefs-checker/invariants/`（`records/2026-10-11-里程碑三知识腐烂治理.md:69`）；这 4 行列进了写回，现在一句都没改，推测是按旧路径找不到就丢了（推的）。
3. **「延后」没有落成欠账行。** #10 的两条改法只写在调度记录 `records/2026-10-13-里程碑四预开发.md:753–754`，`checks-owed.md` 里没有对应一行，后面没有东西提醒，到今天都没做。
4. 机械层的桶名对效果判得不准：#23、#31 桶是 report-unreferenced，实际落了 15/15、10/12；回扫员多半由主 agent 按目录或 spec 批量写回，「没点到报告路径」不等于没用。

## 没做什么

- 没读子 agent 报告的全文：报告还在的 4 份只用了交回；报告不在的从会话记录里抽写报告那几步，生成脚本之后又被同一脚本改过的（#26 的 `L[k]` 两处）按最后一次取。
- #28 的 334 行没逐行核：spec 已不在，只用主会话里写回那一步的原样输出与记录文件第 54 行。#20、#27 的 17 + 12 行半还句只核了「已有 / 仍欠」「去向：」这些标记词在不在，没逐句对拟句。
- 没核「已改」的新句今天是不是还对（只核旧句不在、或新句在）。#22 交现查的 6 条里带日期快照与 C58 去向两条没逐一核。
- 主会话里子 agent 结束之后的动作只按报告路径与关键词筛，没整段通读；记录文件里写回批次的去向按记录原文认。
- 没跑重型测试，没编译，没提交。

## 草稿与产物

草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-investigator-sweep/`：`hb/`（32 份交回原文）、`rep/`（14 份回扫员写报告的步骤）、`tools/sub.py`、`tools/main.py`、`tools/sweep-check.sh`、`sweep-rows.tsv`、`evidence-investigators.txt`、`evidence-sweeps.txt`。没有仓副本、编译目录或工作树。
- ad3a0785a7bcebd5826334bb25f22a8b43e70cac3262135fb8e89218250ba4fe  evidence-investigators.txt
- 5f77cf9eaeee7f64a6673f1c1bf40e84c490f18dbf4f4d1c121facb2d2331a81  evidence-sweeps.txt
- e0e4bf08e31bbf3f740faf63d3b483d3faaf4127a94140ba058fda4ca1433c46  sweep-rows.tsv
- 7d18afdeb5201a996f5b8f0dd988ad77d7b1bdf26f8e5c432a55494227dbe1d3  tools/sweep-check.sh
