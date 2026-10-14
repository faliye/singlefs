# m3-batch62-code-r1 正推腿报告（云端正推，Sonnet）

立场：核代码与测试做的是不是判决 `research/prompts/m3-batch60-61-code-r1-main-verification.md` 第三节 Q1–Q4 点名的那件事。代码行号按冻结副本 `~/.local/share/singlefs/work/m3-batch62-code-r1/repo/`（下文路径省去这个前缀）。
没有起 cargo、没有跑任何测试二进制（checker 档全是重型；GPU 不起）；下面「读码推出」与「命令量出」分开标。

草稿：`~/.local/share/singlefs/work/m3-batch62-code-r1-sonnet/`（`anchors.py` 核变异锚点命中次数；`tally.py` 手算 28 行 / 7 批的小模型，见 R3）。

## 判决原文（第三节 Q1–Q4，逐字，`m3-batch60-61-code-r1-main-verification.md` 的行）

以下四条整行抄自 `research/prompts/m3-batch60-61-code-r1-main-verification.md`（行号用 grep 现取：Q1 在第 37 行、Q2 第 38 行、Q3 第 39 行、Q4 第 40 行）：

| Q1 | 第 61 批流水线导入用例（或新一条同形）设 `SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu`，断言导入那一趟 `blocks_without_verifier == 0`、按块读回的核对行与整库导入（或一台判完那份库）相同；取攻方 h02 那种「第二台那一块只在对方库里核过」的形状 | 变异表追加攻方 `mv`（`crash_amplification.rs` 排键处两种核对代码都不点）那一行，点名新断言、证红 | P4-1 |
| Q2 | 第 61 批两条用例各在导入之后再导一次，钉第二次的 `imported_blocks` 等于第一次（全是已有且相同） | 攻方 `mi-pipe`、`mi-given` 两行进变异表、证红 | P5-2 |
| Q3 | 按计划导入的换批：批长做成可由测试给的参数（产品默认 64 MiB 不变），用例在小批长下走多批，断言跨批编号接续、读回与一批相同；再造一份事实行缺的本库（核对行在、事实行不在）重导，断言事实行补上 | 至少三行变异：换批判据反过来、跨批编号重起、`facts_digests_named.insert` 挪到跳过判之后；各点名、证红 | 正推 B1、B2 |
| Q4 | GPU 配置下点名导入：给定状态表那一路与流水线那一路，漏掉 GPU 判器那一种结论、漏掉 GPU 核对代码那一把摘要，各有一条带 `gpu` 特性的用例会红 | 两行变异各点名、卡上证红（主 agent 照清场规则排卡） | 第 61 批第二位交回、攻方 P4-1 第 4 句 |

## R1　核对行钉得住没有（Q1）——判：对

Q1 点名的是三件事：流水线导入那一趟设 `VERIFY=cpu`、断言缺口计数为 0 加核对行读回相同、取 h02 形状；变异是「排键时两种核对代码都不点」。

**逐步数哪一块在哪台核**（用例 `crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1219`，读码推出）：

| 步 | 环境 | 库 | 判 | 核（VERIFY=cpu） |
|---|---|---|---|---|
| 1 `one_host_run` | 无 SHARE | one_host_library | 块 0、1、2 | 块 0、1、2 |
| 2 `other_share_run` | SHARE=1/2 | other_share_library | 块 1 | 块 1，只在这份库里 |
| 3 `this_share_run` | SHARE=0/2 | this_share_library | 块 0、2 | 块 0、2；块 1 没判（用例自己断言 `unjudged_blocks` 为 1） |
| 4 `importing_run` | 0/2 加 IMPORT=other_share_library | this_share_library | 不判 | 只核 `selected`（块 0、2）；块 1 的结论与核对行只能靠导入 |

依据：`VERIFY=cpu` 在第 1252 行设进整条环境，四趟（五趟，含第 1275 行再导一次）都带；第 ② 段只核本机领的块——「&|block| selected.contains(&block),」（`crates/singlefs-checker-tier/src/crash_amplification.rs:8130`）在 `verify_blocks_from_facts` 调用里；第 ③ 段比对却走计划里每个点的每一块——`compare_the_conclusions_and_the_verifier` 里「comparison.blocks_without_verifier += 1;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3675`）对「有结论、没核对行」的任何块计数，不分领不领。所以块 1 在步 4 的比对里有没有核对行，只看排键点没点它。h02 形状（第二台那一块只在第二台的库里核过）在步 2、3、4 里真的出现了。

**两条断言各自只有点全才过吗**（读码推出，没跑）：
- 断言一：「导入那一趟第 ③ 段：第二台那一块的核对行随结论一起导进来，没有一块缺核对行」（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1289`）钉 `(blocks_without_verifier, blocks_without_judge, disagreements) == (0,0,0)`。变异「排键时一行核对行都不点」把 `for verifier_digest in &verifier_digests {`（`crates/singlefs-checker-tier/src/crash_amplification.rs:7770`）换成 `.iter().skip(len)`，计划里 `verifier_blocks` 为空，块 1 的核对行不进库，步 4 的比对 `blocks_without_verifier` 为 1，红在这一条，而且是用例里排最前的断言。
- 断言二：用例末尾的循环（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1364` 起）对 3 块逐个读 `read_verifier_block`，要求本机库与一台判完的那份相同，并读回核对行点名的事实行。同一变异下块 1 的核对行读回是 None，与一台判完的 `Some` 不等，红在「块 {} 的核对行：导入之后的本机库与一台判完的读回不同」（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1388`）。
- 两条互不依赖：断言一读的是比对结果，断言二读的是库里的行；去掉任何一条另一条仍红。所以「核对行那一条从来没起作用」不成立：变异下两条都起作用，红的第一处是断言一。
- 只漏一种核对代码：非 `gpu` 特性编译时 `verifier_digests` 只有 CPU 一把（`crates/singlefs-checker-tier/src/crash_amplification.rs:7714` 那一把 GPU 摘要在 `cfg(feature = "gpu")` 之下），所以 CPU 用例里「两种都不点」与「只漏 CPU 那一把」是同一个结果；GPU 那把归 R4。

**判决原文对照**：Q1「断言导入那一趟 `blocks_without_verifier == 0`、按块读回的核对行与整库导入（或一台判完那份库）相同；取攻方 h02 那种「第二台那一块只在对方库里核过」的形状」（`research/prompts/m3-batch60-61-code-r1-main-verification.md:37`）。用例三件都做了：缺口计数不只 `blocks_without_verifier`，连带 `blocks_without_judge`、`disagreements` 一起钉 0；读回比的是「一台判完那份库」，并多读了事实行。变异那一行：判决要「点名新断言、证红」，变异表第 3986 行点名的就是这条用例。

推翻条件：(a) 冻结副本上跑这条用例，基线红；(b) 把第 7770 行改成 skip 之后这条用例仍然绿（主 agent 证红 7 行里第 1 行没抓到）；(c) 步 3 里 `this_share_library` 其实已经有块 1 的核对行（`SINGLEFS_CRASH_AMPLIFICATION_IMPORT` 在步 3 被意外带上）。主 agent 手上的证红能直接判 (b)。我复核不了：基线是否绿、变异是否真红（checker 档，我不起）。

## R2　重导计数（Q2）——判：对

**口径**：二选一里是「数点名到、且对方库里有的块」，不是「数写下的块」。两处都是同一个式子：
- 流水线：「tally.conclusion_blocks_written + tally.conclusion_blocks_already_identical;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:7787`），累进 `EnvironmentRun.imported_blocks`（字段在 `crates/singlefs-checker-tier/src/crash_amplification.rs:7191`）。
- 给定状态表：同一个式子在第 8910 行，累进账 `GivenStatesRun.tally.imported_blocks`（字段在第 8374 行），汇总行里的 `imported_blocks=` 只在给定状态表那一行（第 8807 行）；流水线这一路没有打印这个数，只是 `EnvironmentRun` 的字段（`rg 'imported_blocks=' crates/singlefs-checker-tier/src/crash_amplification.rs` 只命中 8807）。
- 对方库里没有的那一项（`conclusion_blocks_missing_in_the_other_library`）不进这个数。

与判决的对照：P5 行「`imported_blocks` 去掉「已有且相同」那一项照绿」（`research/prompts/m3-batch60-61-code-r1-main-verification.md:28`）说的就是这个式子里的 `conclusion_blocks_already_identical`；Q2「钉第二次的 `imported_blocks` 等于第一次（全是已有且相同）」（`research/prompts/m3-batch60-61-code-r1-main-verification.md:38`）。变异 `+ 0 * tally.conclusion_blocks_already_identical` 在第一趟（全写下）报 2，第二趟（全已有）报 0；用例的 `(importing_again_run.imported_blocks, …) == (importing_run.imported_blocks, 0, 0)`（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1297`）加上绝对值「importing_run.imported_blocks, 2,」（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1335`）合起来钉第二次也是 2，变异下 0 不等 2，红。给定状态表那条同理：四元组 `(imported_blocks, tables_reused, tables_judged_here, tables_left_to_the_other_hosts)` 第一趟钉 `(4, 4, 0, 0)`（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1561` 那一行的 `(4, 4, 0, 0),`，断言从第 1559 行起），第二趟 `tally_counts(&importing_again_run),`（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1565`）与它相同；变异 `0 *` 使第二趟成 `(0, 4, 0, 0)`，红。

**第二次导入真的会再走导入吗**：给定状态表一路，「self.import_the_table_from_the_other_hosts(&key)?;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:9399`）在判前、复用检查之前，每趟对每张不归本机的表都调一次，所以第二趟 `imported_blocks` 不会因为「库里已经有了就不导」而成 0；流水线一路是第 7777 行起的 `for directory in list.split` 循环，对每份要导入的库都调 `import_the_planned_rows_from`，不看本库里有没有。

**四元组「与第一次相同」有没有钉上计数以外的东西**：钉上了 `tables_reused`、`tables_judged_here`、`tables_left_to_the_other_hosts` 和交回的结论逐张相同（`after_importing_again == after_the_named_import`）。正确实现下这几项第二趟都不变：每张表仍复用、仍不现判；结论的 `source` 只有 `JudgedOrReadBackOnThisHost` 与占位两种（`crates/singlefs-checker-tier/src/crash_given_states.rs:113`），两趟都是前者。我没有找到正确实现下会因无关变化而红的路径（时间戳、序号、库路径都不在这四个数与结论里）。流水线一路多钉了 `checking.blocks_checked == 0` 与 `unjudged_blocks == 0`，第一趟同样已断言为 0（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1313` 与 `crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1317` 两行），第二趟与它同因。
推翻条件：正确实现下第二趟 `tables_reused` 不是 4（比如复用判据读了「本趟是否现判」）；或第二趟导入因 `StoredIdentically` 之外的比较结局写了东西使 `blocks_checked` 非 0。

## R3　换批、跨批编号与事实行补写（Q3）——判：对（带一条读码推出的旁注）

**私有批长参数只有单测能给吗**（命令量出）：
```
$ cd <冻结副本> && grep -rn 'import_the_planned_rows_in_batches_of' crates --include=*.rs | cut -c1-150
crates/singlefs-checker-tier/src/verdict_store.rs:1777:        self.import_the_planned_rows_in_batches_of(other, planned, PLANNED_IMPORT_BATCH_BYTES)
crates/singlefs-checker-tier/src/verdict_store.rs:1785:    fn import_the_planned_rows_in_batches_of(
crates/singlefs-checker-tier/src/verdict_store.rs:4170:            .import_the_planned_rows_in_batches_of(
```
（上面是这一次真跑出来的全部三处命中：一处产品入口调用、一处定义、一处单测调用。）它没有 `pub`，只有同模块的测试能调；产品只有 `import_the_planned_rows_from`（`crates/singlefs-checker-tier/src/verdict_store.rs:1777` 把批长钉成 `PLANNED_IMPORT_BATCH_BYTES`，常量是 64 << 20，在第 1024 行）。`import_the_planned_rows_from` 的两处产品调用方（流水线 `crates/singlefs-checker-tier/src/crash_amplification.rs:7784`、给定状态表 `crates/singlefs-checker-tier/src/crash_amplification.rs:8902`）都不带批长。环境变量里没有批长的读法：`grep -n 'BATCH' crates/singlefs-checker-tier/src/crash_amplification.rs` 与 `grep -n 'env::var' crates/singlefs-checker-tier/src/verdict_store.rs` 这一次都是 0 行输出。所以产品路径上拿不到非 64 MiB 的批长，判决要的「产品默认 64 MiB 不变」成立。

**28 行 / 7 批手算**（读码 + 小模型）：照 `import_the_planned_rows_in_batches_of` 的循环次序（结论 → 路径 → 核对行 → 事实）逐项数。每块一种结论的行数：块 1 + 违例数 + 计数向量 1 + 块属性 1 + 恢复字（有则 1），见 `put_conclusion_rows_into_the_batch`（`crates/singlefs-checker-tier/src/verdict_store.rs:934`）。

| 批 | 内容 | 行 |
|---|---|---|
| 1 | F1 GPU（1 违例，有恢复字）：1+1+1+1+1 | 5 |
| 2 | F2 GPU（新向量 `red-f-judged-on-the-other-host`，编号 2，两张表各一行）：5+2 | 7 |
| 3 | F2 观察者（无恢复字）：1+1+1+1 | 4 |
| 4 | F3 GPU（新向量 `red-f-block-three-on-the-other-host`，编号 3）：5+2 | 7 |
| 5 | 2 条路径（放完不判批长）加 F1 的 CPU 核对行 | 3 |
| 6 | F2 的 CPU 核对行 | 1 |
| 7 | 事实行 | 1 |

5+7+4+7+3+1+1 = 28，7 批。账的其余格：F1 CPU 本库已有且相同 1；对方库里没有的 4（F1 观察者、F2 CPU、F3 CPU、F3 观察者）；路径写 2（`f.0` 身份相同跳过，`f.0/f.1` 身份不同、`f.0/f.1/f.2` 新）；核对行写 2（F1、F2 的 CPU 摘要；GPU 摘要对方库没有、F3 两把都没有）；事实 1。与用例里的钉值一致（`synced_writes: 7,`，`crates/singlefs-checker-tier/src/verdict_store.rs:4200`；产品批长一份是 `synced_writes: 1` 在第 4186 行）。
脚本版：`research/prompts/m3-batch62-code-r1-sonnet-model/tally.py`，跑出来原样：
```
1 字节一批: (4, 1, 4, 2, 2, 1, 28, 7, [5, 7, 4, 7, 3, 1, 1])
64MiB: (4, 1, 4, 2, 2, 1, 28, 1, [28])
```
这个脚本把我读码的结果写成了程序，它和读码不是两条独立的路；独立的一条要靠主 agent 跑基线（用例一跑就知道）。两个能让手算错的前提我逐个核了：刷批的检查只在紧跟一次放入之后（结论 1842 行、核对行 1904 行、事实 1922 行三处调用），路径循环里没有，所以不会拆开路径行；刷批后换上的空批在下一次放入之前不会再被检查，所以不会写出空批。

**「逐字节相同」与「编号 2、3」能抓住编号只在一批之内接续的错法吗**（读码推出）：编号靠 `registered_count = self.verdict_vector_by_number.len()`（`crates/singlefs-checker-tier/src/verdict_store.rs:2023`）加这一批里已登记的个数；上一批写下之后要把那一批登记的向量推进内存表，才有下一批的起点——这一步是 `write_the_planned_import_batch` 里「for verdict_vector in pending.verdict_vectors_registered_in_the_batch {」（`crates/singlefs-checker-tier/src/verdict_store.rs:2069`）。变异 5（`.into_iter().take(0)`）让这个循环空转：F2 那批写下后内存表还是 2 个，F3 的新向量拿 `2 + 0 = 2`，与 F2 那种撞号；`VerdictVectorContentByNumber(2)` 被改成 F3 的内容。两份库的整库条目不再相同，红在「多批与一批写下的库逐字节相同（跨批登记的判定向量编号接着上一批往后编）」（`crates/singlefs-checker-tier/src/verdict_store.rs:4207`）；即使这一条被去掉，重开之后「编号 2」读到的是 F3 的内容，与期望的 F2 内容不等，也红。所以「编号只在一批之内接续」这类错在这条用例里两条独立的断言都会红。
这条用例的 F2、F3 的新向量刻意放在两批里（批 2、批 4），这是它抓得住的原因；若两种新向量落在同一批，这类错抓不住——这里不是。

**事实行补写是产品里会出现的形状吗**：会，条件要大批。`import_the_planned_rows_in_batches_of` 的写次序是结论、路径、核对行、事实；事实放在最后（循环在 `crates/singlefs-checker-tier/src/verdict_store.rs:1909`），核对行写下之后若中途满批刷出而事实行还在攒着的批里，此时进程被杀或报错，库里就是「核对行在、事实行不在」。64 MiB 批长下，要结论加核对行攒够 64 MiB 才会中途刷出（一块 GPU 结论约 2 MiB，注释在第 1022 行），几十块的流会遇到；这就是判决 B2 说的「断了重跑」那一形。重跑时计划里同样的核对行键又来一遍，核对行与对方相同走 `continue`，而 `facts_digests_named.insert(facts_digest);`（`crates/singlefs-checker-tier/src/verdict_store.rs:1893`）排在跳过判（第 1898 行）之前，事实摘要照记，第 1909 行起的事实循环把缺的补上。用例（`crates/singlefs-checker-tier/src/verdict_store.rs:4249`）造的是这个形状：本库先写 F1、F2 的 CPU 核对行（与 B 逐字节相同，用例自己读回对比），没有 F 的事实行；计划只带核对行（结论与路径空）。重导只点核对行就补得齐：事实循环只看 `facts_digests_named`，与结论、路径无关；账钉 `flow_facts_written: 1, rows_written: 1, synced_writes: 1`（第 4296 行 `synced_writes: 1`），事实行读回 `b"facts of flow f"`。变异 6 把 insert 挪到跳过判之后，核对行相同就 `continue` 先走掉，事实摘要集合为空，账是 0/0/0，红在用例里排最前的那条 `assert_eq!(tally, …)`。

**整行替换那一行在新写法下还改不改行为**：改。原文 `if pending.batch.size_in_bytes() < batch_size_in_bytes {`（`crates/singlefs-checker-tier/src/verdict_store.rs:1943`）是「没到批长就返回 None、不刷」；改成 `>=` 之后，没到批长（产品入口传 `PLANNED_IMPORT_BATCH_BYTES`，一批的字节数远小于 64 MiB）反而往下走，每放一组就刷一批。第 61 批那条 21 行用例（点名的测试，账里钉 `synced_writes: 1`，在 `crates/singlefs-checker-tier/src/verdict_store.rs:4038`）会得到多批，红。变异表第 3965 行的锚点原文、点名测试都对（`research/prompts/m3-batch62-code-r1-sonnet-model/anchors.py` 数过：第 3965 行与第 3986–3994 行共 10 行，原文在各自的文件里各恰好命中 1 次，输出列在 R5）。

**对判决 Q3 的逐条对照**：Q3 要「批长做成可由测试给的参数（产品默认 64 MiB 不变），用例在小批长下走多批，断言跨批编号接续、读回与一批相同；再造一份事实行缺的本库（核对行在、事实行不在）重导」（`research/prompts/m3-batch60-61-code-r1-main-verification.md:39`）与「至少三行变异：换批判据反过来、跨批编号重起、`facts_digests_named.insert` 挪到跳过判之后」。代码都做了；三行变异加一行整行替换（共四行）；用例比判决多了一条：重开之后每块每种结论按内容读回与 B 相同。

**旁注（读码推出，没造，零轮，不算打中 Q3）**：`write_the_planned_import_batch` 里推进内存表有两处（`verdict_vector_by_number.push` 与 `verdict_vector_number_by_content.insert`），变异 5 只让两处一起不执行。若只去掉 `number_by_content.insert`，同一次导入里不受影响（每个对方编号只换一次，由 `this_number_of_each_other_number` 记着），本用例重开后从盘上重建，也看不出；影响的是同一个打开着的库接着导第二份对方库（流水线的 `IMPORT=a:b` 循环，`crates/singlefs-checker-tier/src/crash_amplification.rs:7777`），第二份里同内容的新向量会被当成没登记过而再登一次。判决 Q3 没点这一把，也没有任何用例钉它。
推翻条件（本节主结论）：基线 28 行 / 7 批算错（跑出来 `rows_written` 或 `synced_writes` 不是 28 / 7）；变异 4、5、6 或整行替换任何一行没被点名的测试抓到。

## R4　GPU 两条与「本次未跑」（Q4）——判：对

Q4 原文：「GPU 配置下点名导入：给定状态表那一路与流水线那一路，漏掉 GPU 判器那一种结论、漏掉 GPU 核对代码那一把摘要，各有一条带 `gpu` 特性的用例会红」（`research/prompts/m3-batch60-61-code-r1-main-verification.md:40`）。新文件两条：流水线 `importing_the_other_share_on_the_gpu_brings_its_gpu_judge_conclusion_and_its_gpu_verifier_row`（`crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs:178`）、给定状态表 `given_states_tables_judged_on_the_gpu_by_another_host_are_imported_with_their_gpu_judge_conclusions`（`crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs:380`）。

**「本次未跑」门禁认不认**（读脚本）：两条用例的未跑行都经 `not_run_line`，其记号是 `NOT_RUN_NO_GPU_ADAPTER`（`GPU_NOT_RUN_MARKER`，`crates/singlefs-harness/src/gpu_memory_ledger.rs:567`），格式 `<记号> test=<名> reason=<原因>`（同文件第 572 行）。门禁 `checker-tier-layer0-replay` 的 feature-gated-tests 格对挂特性的测试日志逐处找这个记号：「done < <(gpu_adapter_not_run_lines "$feature_log")」（`.claude/gate.d/checker-tier-layer0-replay.sh:792`）里每一处调 `layer0_report_not_run`，它打「  ! 本次未跑（这不是通过）：$1」（`.claude/gate.d/checker-tier-layer0-replay.sh:570`）；有一处就把格状态记成「feature_cell_state="有 ${not_run_total} 处本次未跑"」（`.claude/gate.d/checker-tier-layer0-replay.sh:796`），不算全绿。另一条路是「count_tests_run_by_file "$feature_log"」（`.claude/gate.d/checker-tier-layer0-replay.sh:775`）数用例条数，提早返回的用例算「跑到 1 条」；单靠它会把「返回了」当成「跑过」，所以格另靠记号那一路兜住，两路合起来才是「跑了且没未跑」。新文件的两条沿用旧的约定（`crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs:173` 同样的「if run.stopped_early {」之后打行返回），所以门禁认；我没有跑门禁（不归我），上面是读脚本。

**三种未跑的分支会不会吞掉真红**（读码推出）：
- 没卡：`gpu_card_pool_or_report_not_run`（`crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs:181`、第 384 行）只在池起不来且判成「没有适配器」时返回 `None`；有卡而用不上是 panic（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 里的 `CardsPresentButNotUsable` 分支），不是未跑。
- 流水线 `stopped_early`（`crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs:216`，三趟任一趟）：`stopped_early` 的来源只有被叫停或显存不够——「let stop_was_requested = gpu_memory_was_short || stop_is_requested();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:8075`）。叫停看 `SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE`（`crates/singlefs-checker-tier/src/crash_amplification.rs:1582`），这个变量在 `ENVIRONMENT_VARIABLES_READ_BY_A_RUN`（`crates/singlefs-checker-tier/src/crash_amplification.rs:7384`）里，用例的 `cleared_then_set` 先清掉，所以调用方环境里残留的叫停文件不会触发。显存不够只来自卡池借不到或卡上报 Out of Memory，是卡上显存被别的进程占着，不是本机 CPU 负载高；这是旧约定（第 10v、10x、10z 批）定的「不够就是本次未跑、不是红」，不是这一批新开的口子。**能被吞掉的**只剩：产品代码把 `gpu_memory_was_short` 错判成真的。这一支的返回发生在所有断言之前，所以那种错判下新用例里的导入断言全不跑；旧的 `the_product_path_judges_on_scratch_it_booked_on_the_card_pool` 同形，同样如此。没有比旧约定更差，但也要写明。
- 给定状态表：`Err(reason) if reason.starts_with(GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX) => {`（`crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs:359`）只吞「运行条件不满足」那一行；别的 `Err` 走下一行「Err(reason) => panic!(」（`crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs:364`）。变异 9 造出的是 `GIVEN_STATES_TABLE_JUDGED_BY_NO_HOST_PREFIX`（`crates/singlefs-checker-tier/src/crash_amplification.rs:8425` 定义的「CRASH_POINT_REPLAY_GIVEN_STATES_TABLE_JUDGED_BY_NO_HOST」，与「…RUN_CONDITION_NOT_MET」不是前缀关系），走 panic，不会被吞。导入那一趟的判器不借显存（每张表导入后即复用），所以那一趟的 `Err` 若出现也是真红。

**2 块与 8 块怎么来、前提断言能不能在错配置下照过**（读码推出）：
- 2：第二台在卡上判块 1，存 GPU 判器结论一块，配置开 GPU 时对照那一遍存 CPU 判器结论一块（经旁库导回主库，`crates/singlefs-checker-tier/src/crash_amplification.rs:9851` 的 `main_store.import_from(&self.store)`），共 2；用例不把 2 写死当预期，而是 `other_share_conclusions.len()` 先断言为 2（前提），再要 `importing_run.imported_blocks` 等于这个长度。
- 8：4 张表 × 两种结论（`settle_the_gpu_judgement` 在 `crates/singlefs-checker-tier/src/crash_amplification.rs:9231` 把 GPU 与对照的 CPU 都存），前提断言先钉 `(tables_judged_here, states_judged_by_the_gpu_here) == (4, 8)`（`crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs:434` 那一行 `(4, 8),`）、库里 GPU 判器结论 4 块、总共 8 块，最后 `(8, 4, 0, 0)`（第 467 行）。
- 错配置：配置没开 GPU 时，这一趟走 CPU 判器，库里不会有 `ConclusionKind::GpuJudge` 的块，前提「第二台那一份库里有 GPU 判器那一种结论」红；`other_share_run.gpu_judge.is_some()`（第 228 行）、核对方式 `Some("gpu")`（第 236 行）也红。所以前提在错配置下是红，不是绿；会绿的只有真的在卡上判了的配置。

**三行 GPU 变异各红在哪一条**（读码推出，没起卡）：
1. 排键时漏 GPU 判器（`crates/singlefs-checker-tier/src/crash_amplification.rs:7736` 的 `JudgedBy::GpuJudge.conclusion_kind(),` 换成 CPU 一种）：计划里的结论成了 [CPU, CPU]，按键去重后块 1 的 GPU 结论不导，库里没有，红在「导入之后 GPU 判器那一种一块没判的都没有：第二台那一块的 GPU 判器结论导进来了」（`crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs:268`）。
2. 排键时漏 GPU 核对摘要（`crates/singlefs-checker-tier/src/crash_amplification.rs:7714` 那一把换成 CPU）：块 1 的 GPU 核对行不导；步 ② 在导入那一趟只核本机领的块，第 ③ 段比对用 GPU 摘要，块 1 有结论没核对行，红在「导入那一趟第 ③ 段拿 GPU 核对代码的核对行比：第二台那一块的 GPU 核对行导进来了，没有一块缺核对行」（第 286 行）。
3. 点名漏 GPU 判器（`crates/singlefs-checker-tier/src/crash_amplification.rs:8884` 的 `conclusions: [JudgedBy::GpuJudge, JudgedBy::CpuJudge]` 去掉 GPU）：导入之后库里没有 GPU 判器结论，引擎判第一张表报 `…TABLE_JUDGED_BY_NO_HOST`，`panic!` 红（第 364 行）。
第 3 行不是判决点名的「两行」之一：判决 Q4 写的「漏掉 GPU 核对代码那一把摘要」在给定状态表那一路不存在——「verifier_blocks: Vec::new(),」（`crates/singlefs-checker-tier/src/crash_amplification.rs:8895`）说明那一路导入从来不点核对行。所以判决 Q4「给定状态表那一路与流水线那一路，漏掉 GPU 判器那一种结论、漏掉 GPU 核对代码那一把摘要」应读成：判器那一种两路各一条，摘要那一把只有流水线一路，共三行；实现员报告说明了这一点，我读到的与它一致。
推翻条件：卡上基线红；三行证红任一行没被点名的那条用例抓到；`rg 'NOT_RUN_NO_GPU_ADAPTER'` 在两条用例的未跑分支之外还打了别的路径。我复核不了：卡上是否真绿真红（本机 NVML 驱动对不上，实现员、主 agent 都记过），门禁 feature-gated-tests 格的真实输出（不跑门禁）。

## R5　改动范围与产品路径——判：对

**`crash_amplification.rs` 没改，是产品本来就对、只差测试吗**（读码 + 判决对照）：
- 快照校验：`sha256sum -c research/prompts/m3-batch62-code-r1-snapshot/crates-sha256.txt`（在冻结副本根跑）这一次五行全 OK，其中 `crates/singlefs-checker-tier/src/crash_amplification.rs: OK`，说明被判的是这一份；改动全文（`research/prompts/_m3-batch62-code-r1-diff.md`）里 `--- a/` 只有 `verdict_store.rs` 与流水线用例，加一个新文件，`crash_amplification.rs` 不在其中。
- 判决点名的缺口各自是什么：P4 行「测试缺口，产品代码在原样下读回相同」（`research/prompts/m3-batch60-61-code-r1-main-verification.md:27`）；P5 行「`imported_blocks` 去掉「已有且相同」那一项照绿」（同文件第 28 行），要改的是不让它照绿，加断言就够；B1「采纳为测试缺口」（第 29 行）、B2「采纳」（第 30 行）。Q4 的缺口也是没有用例。
- 产品代码里每个被钉的行为，我在冻结副本上读过，原样就是判决要它是的样子：排键点两把核对摘要（第 7770 行循环）、导入块数 `written + already_identical`（第 7787 行、第 8910 行）、`facts_digests_named.insert` 在跳过判之前（`crates/singlefs-checker-tier/src/verdict_store.rs:1893`）、点名 GPU 与 CPU 两种结论（第 8884 行）。唯一要改的产品文件是 `verdict_store.rs`（批长做成参数），这是判决 Q3 自己要求的（「批长做成可由测试给的参数（产品默认 64 MiB 不变）」），不是 `crash_amplification.rs`。
- 没有一处要改产品代码而这一批没改的：我没有找到。

**`verdict_store.rs` 签名变化的影响**：`write_the_planned_import_batch_when_full` 是私有 `fn`，加了 `batch_size_in_bytes` 参数，调用点只有三处（第 1842、1904、1922 行），都改了（diff 里三处）；`import_the_planned_rows_from` 的签名一字不变，转调新私有函数并传 64 MiB 常量。旁库导回主库那一处用的是旧整库导入 `main_store.import_from(&self.store)`（`crates/singlefs-checker-tier/src/crash_amplification.rs:9851`），签名在 `crates/singlefs-checker-tier/src/verdict_store.rs:1681` 没动。别的 crate 里没有这几个函数的调用（上面 R3 的 grep 是 `crates` 全树，仓里 `scripts/` 与 `.claude/gate.d/` 里也没有引用，命令是 `grep -rln 'import_the_planned_rows' scripts .claude/gate.d`，这一次没有输出）。

变异表锚点（命令量出）：`python3 -I research/prompts/m3-batch62-code-r1-sonnet-model/anchors.py <冻结副本>` 原样输出（列：行号、字段数、原文命中次数，截去名字）：
```
3965 6 1
3986 6 1
3987 6 1
3988 6 1
3989 6 1
3990 6 1
3991 6 1
3992 6 1
3993 6 1
3994 6 1
```
十行（整行替换 1 行加追加 9 行）字段数都是 6，原文在各自的文件里都恰好命中 1 次。
推翻条件：有一处调用 `import_the_planned_rows_from` 或 `write_the_planned_import_batch_when_full` 的代码没在上面列出（主工作区此刻的 `crates/` 与冻结副本不同时才可能）；或判决里 Q 条要的某个产品行为在 `crash_amplification.rs` 里其实是错的（用例基线红）。
## 补记（读码时多看到的，不改上面各格的判定）

1. **R1 的事实行读回在 CPU 流水线用例里不起分辨作用**（读码推出）：事实表是整条流一份、一个摘要；每一趟带核对的运行，第 ② 段先无条件存它——「.store_flow_facts(&facts_digest, &facts.to_bytes())」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3537`），哪怕这一趟没有一块要核。所以步 3、步 4 自己就把事实行写进了 `this_share_library`，用例末尾那条事实行比对「块 {} 的核对行点名的事实行：导入之后的本机库与一台判完的读回不同」（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1403`）即使导入不带事实行也照过。Q1 点名的是核对行，核对行那两条仍成立；事实行是否随导入补上，由 Q3 的单元用例（R3，变异 6）钉，不由这条流水线用例钉。这不是 Q1 要的东西，不算打中，也不改 R1 判定。
2. **R3 的「断了重跑」形状的前提要补一句**：本机重跑那一趟若核（开了 GPU 或设了 `VERIFY=cpu`），第 ② 段会重存事实行（上一条），事实行缺口自己补上；导入补写事实行成了唯一补法的情形，是重跑那一趟不核而对方机器核过（两台配置不同）。形状真会出现，只是比 R3 正文写的窄。
3. **R3 旁注**（`number_by_content.insert` 只有同库接着导第二份对方库时才用得到，Q3 用例看不出）见 R3，零轮、读码推出，没造反例。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| R1 核对行钉得住 | 对 | 步 1–4 数下来块 1 的核对行只能靠导入进本机库（h02 形状在用例里真出现）；变异「两种核对代码都不点」红在比对断言（缺 1 块），读回断言同样红，两条互不依赖；事实行读回不分辨（补记 1），不影响 Q1 |
| R2 重导计数 | 对 | 钉的是「点名到且对方有」的块数 = 写下 + 已有且相同，与 `crash_amplification.rs:7787`、`:8910` 同一个式子，也是 P5-2 的量；流水线的 `imported_blocks` 只是字段，`imported_blocks=` 只在给定状态表那一行打；四元组与第一次相同在正确实现下不会因无关变化而红 |
| R3 换批、编号、事实行 | 对 | 批长参数产品拿不到；28 行 / 7 批逐批数对（5,7,4,7,3,1,1）；变异 5 让第二批的新向量撞号，逐字节相同与「编号 2、3」两条独立断言都红；事实行补写形状真会出现（比正文窄，补记 2）；整行替换在产品批长下仍改行为；旁注一条零轮缺口（`number_by_content`） |
| R4 GPU 两条与未跑 | 对 | 未跑行带 `NOT_RUN_NO_GPU_ADAPTER` 记号，feature-gated-tests 格逐处报本次未跑、不判通过；`stopped_early` 只来自叫停（用例清掉环境）或显存不够，旧约定同形；其余 `Err` 走 panic 不被吞；2、8 是用例自己列库核过的前提，错配置下前提红；三行变异各红在点名的断言；卡上真绿真红复核不了 |
| R5 改动范围与产品路径 | 对 | `crash_amplification.rs` 的校验和与快照一致、不在改动里；判决点名缺口都是测试缺口，唯一产品改动 `verdict_store.rs` 是 Q3 自己要的批长参数；私有函数调用点三处全改，旧 `import_from`（旁库导回）未动 |

## 没做什么

- 没起 cargo、没跑任何测试二进制（checker 档全是重型，GPU 不起）；所有「红在哪一条」「基线是否绿」都是读码推出，写作「读码推出」；唯一量出来的是 grep、`sha256sum -c`、`tally.py` 与 `anchors.py` 的输出。
- 没跑门禁阶段；门禁 feature-gated-tests 格怎么认「本次未跑」是读 `.claude/gate.d/checker-tier-layer0-replay.sh` 得出的。
- 没造反例二进制：「编号撞号」「`number_by_content` 缺口」「事实行读回不分辨」都是读码推演，没有在小模型里重做一遍；`tally.py` 把我的读法写成程序，与读码同源，不算独立的第二条路径（见 R3）。
- 没替主 agent 采纳或出判决；上面的「对」是「代码与测试做的是判决点名的那件事」，不是「用例基线绿」。
- 已知欠账（C786、秒数没量、改法 2、C784、形式化轻型红例账缺失）按正文第二节不当作打中，没报。
- 要主 agent 跑的命令：没有新增；证红 7 行（CPU）与 GPU 3 行、两条 GPU 用例基线，照实现员报告「要主 agent 跑的命令」一节与正文第二节写的卡恢复后的安排。

## 草稿与模型

- 草稿目录：`~/.local/share/singlefs/work/m3-batch62-code-r1-sonnet/`（`anchors.py`、`tally.py` 的原件；没有仓副本、没有编译目录，没有要删的东西）。
- 模型目录：`research/prompts/m3-batch62-code-r1-sonnet-model/`：`tally.py`（28 行 / 7 批的小模型）、`anchors.py`（变异表锚点命中次数，用法 `python3 -I anchors.py <冻结副本根>`）。
