# m3-batch62-code-r1 判决：删除线第 62 批代码轮第一轮（2026-10-11）

被判的改动（按路径点名）：`crates/singlefs-checker-tier/src/verdict_store.rs`、`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs`、`crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs`、`crates/mutations.tsv`。

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 R1 R2 R3 R4 R5 S1 S2 S3 G1 G2 G3 G4 G5 G6 -->

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | 冻结副本里 `sha256sum -c research/prompts/m3-batch62-code-r1-snapshot/crates-sha256.txt`；主工作区 `sha256sum -c …/kb-sha256.txt` | 被判的五份都对上。kb 两份对不上：`.claude/kb/checks-owed.md`（别的会话改行号引用、主 agent 追加 C779–C786）与 `.claude/kb/milestone/03-third-txn.md`；两条腿的报告都没引这两份文件的行（`grep` 两份报告里的 `checks-owed.md`、`03-third-txn.md` 零命中），判决不受影响 |
| 正推腿 `research/prompts/m3-batch62-code-r1-sonnet-output.md`（175 行，sha256 f51aebf1cc5d5c56…）、攻方 `research/prompts/m3-batch62-code-r1-opus-output.md`（245 行，sha256 76097916a86beba1…） | `cite-check.py` 两份报告，`--root` 取冻结副本与仓根，`--background` 取背景材料 | 「✓ 核了 79 处引文，对上 79 处，没判 0 处」 |
| 攻方打中的几处代码 | 冻结副本里现读 | `verdict_store.rs` 第 2069–2076 行写完一批之后逐个把这一批登记的向量放进内存表，编号取内存表长度；Q3 用例第 4209 行 `drop((imported_in_one_batch, imported_in_many_batches));`，丢掉之前不经同一个库读回；`crates/mutations.tsv` 第 3965 行与第 3989 行原文同为 `        if pending.batch.size_in_bytes() < batch_size_in_bytes {`、替换文同为 `>=`；`crash_amplification.rs` 第 3537 行第 ② 段开头无条件 `store_flow_facts`。都与腿报的一致 |
| 正推旁注 `number_by_content` | 冻结副本里现读 | `crash_amplification.rs` 第 7777 行 `for directory in list.split(':')` 在同一个打开着的库上逐份导入；`verdict_store.rs` 第 2019 行按内容查编号走 `verdict_vector_number_by_content`。只去掉第 2074 行那一句 `insert` 时，同一次导入里靠 `this_number_of_each_other_number`（第 1797 行）不受影响，接着导第二份对方库时同内容的向量会再登一次：读码推的，没造 |
| 攻方模型 | 拷编译目录种子到 `~/.local/share/singlefs/work/m3-batch62-code-r1-verify/`，`rerun.sh <它> all` | 见第六节 |
| 合入后验证（主 agent 在主工作区跑） | `merge-and-verify.sh --memory 24G --threads 31 <第 62 批补丁目录>` | 「✓ 测试全绿（194 个测试二进制，共 2019 条测试）」，fmt、clippy、build 过，日志 `~/.local/share/singlefs/work/merge-and-verify/2026-10-11-869192` |
| 第 62 批 checker 档用例与 CPU 证红（实现员副本） | 两个测试二进制点名、`prove-red.sh` 7 行 | 「2 passed」「4 passed」；「✓ 点名 7 条：跑了 7 条，抓到 7 条，本次未跑 0 条」 |
| GPU 基线、GPU 证红 3 行、变异轻型 10 行 | 同一份脚本 `run-batch62.sh` | 三步都退 78：清场日志「QUIESCE_STOP_COMMAND 退 1 / nvidia-smi-failed」，本机驱动内核模块 595.84 与用户态库 595.99.02 对不上。不是代码红；卡恢复后同一条命令续跑，结果补进第四节 |

## 二、逐格判

| 格 | 正推 | 攻方 | 判 |
|---|---|---|---|
| R1 核对行钉得住（Q1） | 对：块 1 的核对行只能靠导入进本机库，h02 形状真出现；`mv` 红在比对断言。补记 1：事实行那一句不分辨 | 主问没打中（读码）；**R1-a 打中（小，读码）**：导入那一趟第 ② 段开头自己写事实行（`crash_amplification.rs:3537`），「核对行点名的事实行与一台判完的相同」不管导入带没带事实都过 | 两条腿各自读到同一件事，主 agent 现读第 3537 行属实。**采纳**：这一句与文件头那半句说多了，不是 Q1 字面没做到（Q1 只要求核对行）。改法 S3 |
| R2 重导计数（Q2） | 对：钉的是「点名且对方库有」= 写下 + 已有且相同，与 `crash_amplification.rs` 第 7787、8910 行同一个式子，也是 P5-2 的量 | 没打中（读码）；三个计数邻居逐个推到红处 | 一致，判对。两条腿都只读码、各抽一次 |
| R3 换批、跨批编号、事实行补写（Q3） | 对；28 行 / 7 批逐批 5、7、4、7、3、1、1；**旁注（零轮，读码）**：只去掉 `number_by_content.insert` 时 Q3 用例看不出 | **打中（量过）**：一批里登记两种以上新向量、导完经同一个库读回的形状没有用例；邻居变异 `take1`（一批里只有第一种进内存表）、`rev`（倒着进）在被判三条单测的镜像上全绿。**R3-b 打中（小，量过）**：第 3989 行与第 3965 行同一个替换，红在「照产品的批长：一批写完」，名字里的「一字节一批」那一句走不到 | 主 agent 复跑攻方模型六份输出逐字节相同（第六节），`take1`、`rev` 下被判镜像照绿、攻方 `attack_in_memory_…` 用例红，属实；第 3965、3989 两行原文替换文逐字相同，属实。正推的旁注与攻方的打中是同一处（写完一批之后放进内存表那几行），各打一种错法。**采纳**：改法 S1（关 `take1`、`rev` 与只去掉 `insert`）、S2（第 3989 行换成只在小批长下改行为的替换）。28 行 / 7 批两条腿各自数对，攻方另用盘上差 27 个新键加 1 个改值核过 |
| R4 GPU 两条与「本次未跑」（Q4） | 对（读码）：未跑行带 `NOT_RUN_NO_GPU_ADAPTER`，feature-gated-tests 格逐处报本次未跑；`stopped_early` 只来自叫停或显存不够；别的 `Err` 走 panic | 没打中（读码）；2 块、8 块、三行 GPU 变异红处都是推的 | 一致。卡上没跑：GPU 基线与证红等卡恢复，见第一节末行 |
| R5 改动范围与产品路径 | 对：`crash_amplification.rs` 不在改动里，判决点名的缺口都是测试缺口 | 没打中（读码 + 量过）：原型五条在产品代码原样下全绿 | 一致，判对：这一批没有一处该改产品代码而没改 |

## 三、采纳的改法（第 63 批）

| 编号 | 改法 | 验收 | 来源 |
|---|---|---|---|
| S1 | `verdict_store.rs` 的 Q3 用例（`planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes`）在 `drop` 之前，经照产品批长导完的那一份库按内容读回 F1–F3 每种结论、与对方库比，再登记一种新向量看它的编号接在后面，重开再比（攻方 G1）；另加一步：同一个打开着的库接着从第二份对方库导入同内容的新向量，断言不另起编号（正推旁注） | 变异表追加三行：攻方 `take1`、`rev`（原文与替换文照 `research/prompts/m3-batch62-code-r1-opus-model/mutants.py`），与「写完一批之后只去掉 `verdict_vector_number_by_content.insert`」；三行各点名改过的用例、证红 | 攻方 R3、正推 R3 旁注 |
| S2 | `crates/mutations.tsv` 第 3989 行（第 62 批 Q3 换批判据那一行）换成只在小批长下改行为的替换，让它红在「一字节一批：同样的 28 行分 7 批写」那一句；名字照旧 | 整行替换一行；证红时看红在那一句（不是「照产品的批长：一批写完」） | 攻方 R3-b |
| S3 | `tests/crash_amplification_pipeline_records_checks_and_reuses.rs` 第 1219 行那条：删掉「核对行点名的事实行与一台判完的相同」那一句断言与文件头注释里对应的半句（事实行随导入补写由 `verdict_store.rs` 的库层用例判） | 删后那条用例照绿；变异表不动 | 攻方 R1-a、正推补记 1 |

不采纳：攻方 G2、G3（一块两种新向量的对方库、放开批长扫）。只做 G2 或 G3 关不了 `rev`（攻方改法表），S1 已经关了三种错法；G3 扫 52 档是攻方在原型上量过的，原样下全绿，不进 checker 档。

## 四、轻型与欠账

- GPU 基线、GPU 证红 3 行、变异轻型 10 行：卡恢复后照 `run-batch62.sh` 续跑，结果补一行在这里。
- 补（2026-10-11，重启修好驱动之后）：GPU 基线「2 passed」、日志里没有 `NOT_RUN_NO_GPU_ADAPTER`；GPU 证红「✓ 点名 3 条：跑了 3 条，抓到 3 条，本次未跑 0 条」（`~/.local/share/singlefs/work/m3-batch62/planned-import-pins/main-runs/`）。`run-batch62.sh` 的变异轻型那一步退 1：它在实现员副本里起，副本只拷了 `crates/`，读不到 `.claude/gate.d/crates-mutation-light-rows.tsv`，起法不对、不是代码红；变异轻型改归提交时的 checker-tier-crates-mutation-replay 整表。主工作区轻型崩溃（第 60–63 批都在里面）：`CRASH_POINT_REPLAY_TOTAL tier=quick dispatched=16 collected=16 green=12 red=3 not_run=1`；红的三项都只红在第二台那一份、本机那一份绿：crash-injection-fast-tier 与 read-faulted-states-never-hold-the-witness-criterion 是第二台的进程把自己认成本机、显卡配置对不上（GPU 那 5 条），unlink-after-the-multi-record-publish 是第二台算的计数向量与段区间对不上；两桩各派一个调查员（`~/.local/share/singlefs/work/peer-gpu-role/`、`peer-count-vector/`），都在双机装置一侧，不是第 62 批改的文件。主工作区的轻型崩溃与轻型变异（第 60、61、62 批与 E12–E14 都在里面）同样等卡：上一趟轻型崩溃「13 项红」每一项都是「cargo test 退 78、红 0 条」，轻型变异退 78，原因同一个。
- 别的会话 2026-10-11 在主工作区跑变异轻型（核心部分加两行）：「抓到 171 条、没红 0 条、无效 0 条」，整道红在 `crates/mutations.tsv` 第 293、754、3284 行的点名测试 1800 秒超时；前两行是删除线交接表第 11 项登记过的，第 3284 行要 GPU。不是第 62 批带进来的。
- 欠账照旧：C786（按计划导入几份库同一块各数一次）、C784（只判最右之前 k 片的变异有限组关不死）；改法 2、拷整库给第二台、导入之后的秒数没量、形式化红例账缺失在 `.claude/kb/milestone/03-third-txn.md`「## 待优化：流程、门禁与装置上踩过的坑」第 22、24 条。
- 正推补记 2（断了重跑形状比正文窄：重跑那一趟若核会自己补事实行，导入补写成唯一补法要两台配置不同）：不改判，记在这里。
- 「没打中」各格都只抽了一次样；R2、R4、R5 两条腿都只读码。按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」，这几格不拿来作「没有问题」的结论：S1–S3 合入后的那一轮代码轮复攻。

## 五、第 63 批派发

一个实现员关 S1–S3（落在 `crates/singlefs-checker-tier/src/verdict_store.rs` 的单测、`tests/crash_amplification_pipeline_records_checks_and_reuses.rs` 与 `crates/mutations.tsv`），交补丁；checker 档测试与证红由主 agent 跑。

## 回看决策

不涉及决策：第 62 批只改崩溃放量判定库的测试与一个只给测试的批长参数（验证装置），不碰盘上格式与任何决策分项的条款。

## 六、复跑核对（攻方模型）

在 `~/.local/share/singlefs/work/m3-batch62-code-r1-verify/` 跑 `bash research/prompts/m3-batch62-code-r1-opus-model/rerun.sh <它> all`（编译目录种子拷自 `~/.local/share/singlefs/work/m3-batch60-61-code-r1-verify/target`），退 0，末尾 compare：「一致 base.txt」「一致 mut-facts-after.txt」「一致 mut-flip.txt」「一致 mut-rev.txt」「一致 mut-take0.txt」「一致 mut-take1.txt」。六份输出与攻方 `expected/` 逐字节相同（`cmp`），sha256 前 16 位：base ffb1651054487192、mut-facts-after 640564ed1413b4ae、mut-flip 8801618174bd32b8、mut-rev 44c7892feb6df608、mut-take0 ad413b8372d41a6c、mut-take1 fedcc04710afbb9f，与攻方报告开头列的相同。`take1`、`rev` 两份里攻方用例的红行：

```
thread 'attack_in_memory_table_after_a_product_batch_import' panicked at crates/opus-probe-b62/tests/batch62_store_probe.rs:188:41:
thread 'attack_two_new_vectors_in_one_batch_then_one_more_batch' panicked at crates/opus-probe-b62/tests/batch62_store_probe.rs:392:60:
thread 'sweep_batch_lengths_against_one_batch' panicked at crates/opus-probe-b62/tests/batch62_store_probe.rs:457:68:
thread 'attack_in_memory_table_after_a_product_batch_import' panicked at crates/opus-probe-b62/tests/batch62_store_probe.rs:366:5:
```

前三行在 `mut-take1.txt`，末行在 `mut-rev.txt`；两份里被判三条单测的镜像（`mirror_*`）都没有红行，与攻方「`take1`「3 passed; 2 failed」」「`rev`「4 passed; 1 failed」」相同。这是原型 crate 上量的，被判的 checker 档用例在 `take1`、`rev` 下照绿是推的，S1 的证红在 checker 档上坐实。
