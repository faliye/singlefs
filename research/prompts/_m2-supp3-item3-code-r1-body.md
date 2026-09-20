# 背景材料：增补 3 第 3 件（崩溃注入）的实现，代码轮第一轮

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 -->

判的是里程碑「第二个事务」增补 3 第 3 件写出来的代码，还没提交。开工快照 `research/prompts/m2-supp3-item3-code-r1-start-snapshot.sha256`（12 个文件，2026-09-20 08:07 UTC 记；腿跑着的时候主 agent 不改这些文件）。diff 与新模块整份在 `_m2-supp3-item3-code-r1-diff.md`。

## 一、这一件要做成什么（里程碑原文）

> 3. 崩溃注入：第 1 件的每段历史按种子抽若干次写，在那次写之后截断（层 0 的录制与截断，`crates/singlefs-harness/src/crash.rs`），恢复之后跑 checker，并问模型这个状态恢复到的版本允不允许。抽样，不全枚举；全量枚举仍归门禁 54 号的两条固定流。

「预想的细节」里压着的五条：已知问题登记成一张「已知红」清单、模型在打回重议的几处照代码今天的保守读法写并标预想、规模分快档与大档两档、发现的问题照主 agent 判阻塞办、捶打不改设计。

## 二、实现今天的样子（主 agent 的观测，读的是工作区那一版）

- 新模块 `crates/singlefs-harness/src/crash_injection.rs`（1044 行）：`inject_crashes_into_history(history, execution, crash_points_per_history)` 跑一段历史、开内容保留的录制流（`SharedStream::retaining_contents`），把「起点跑完时流里有几步」与「最后一个跑完的操作之后有几步」记成 `StreamMarks`，崩溃点只抽在这段区间里；每个崩溃点截断录制流、用 `MemoryPool` 重建镜像、`recover(&image, JournalPolicy::Consult)`、跑池级 checker、问模型。`run_crash_injection_campaign` 按种子区间切片并行，线程数从 `SINGLEFS_CRASH_INJECTION_THREADS` 取、没设取 `available_parallelism`。
- 判读那一步取的是 `model_comparison::observed_read_back_after_a_crash(&report)`：根取 `RecoveryReport::effective_root`（施加 journal 记录前缀之后**实际走的**那条根），不取 `RecoveryOutcome` 里带的那条。
- 模型那一侧：`IdealModel::committed_versions()` 把此刻根环里每条根与它下面的内容摘出来，崩溃注入每一步之后并一次，攒成「模型提交过的每一版」的目录；`crash_recovery_disagreement` 判两件事——择到的根要是模型提交过的某一版、那一版的内容要与实现读回的逐字节相同。
- 用例 `crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs` 五条：快档（24 段 × 24 步 × 每段 4 个崩溃点，`GenerationWeights::BROAD`）、一段写死历史的全部崩溃点、崩溃点要落进回退与抬 F、线程数 1 与 4 报告逐字相同、抽崩溃点不改变这段历史怎么跑；大档 `#[ignore]`、按 `SINGLEFS_CRASH_INJECTION_*` 从环境变量取规模。
- 快档一轮的读数（2026-09-20，release）：24 段历史、92 个崩溃点（截在发布中间 77 个、截回到最后一版之前 85 个），恢复读回文件 61 次 / 没有文件 31 次 / 失败 0 次、施加过 journal 记录前缀 14 次，崩溃点上 checker 跑 92 次、问模型 92 次（比过内容 58 次、树表 0 条对上 31 次），崩溃状态以已知红第 0 条收尾 2 个、新发现 0。
- 第一次跑抓到 4 条「冷启动读回」对不上，全是截在「journal 记录已写、根槽未写」那个窗口。主 agent 核下来：实现照 D23（journal 的角色与格式） 已定项 14 的前缀口径施加了记录，内容前进到在飞那次发布的内容而根身份还停在旧根；判读那一步当时取的是择根，所以配不上。接上 `observed_read_back_after_a_crash` 之后四条全消。坐实的数：载荷容量 32634 下，种子 1 的操作 0 = 16599 字节（模型答的）、操作 1 = 22950 字节（实现读回的）。
- 变异表 `crates/mutations.tsv` 新增两行（整表 175 条）：把「抽 0 个崩溃点」改成照样抽一个 ⇒ `drawing_crash_points_does_not_change_the_generated_history` 红；把判读改回 `observed_read_back` ⇒ `crash_injection_fast_tier_recovers_only_into_versions_the_model_committed` 红。
- 门禁：`check.sh` 四段绿、74 号 exit 0（五段都判过）、33 号锚点各命中一次、命名纪律绿、76 号绿、doc-lint 443 项绿。

## 三、要判的问题

**K1（抽样罩得住吗）**：崩溃点只抽在「起点跑完」到「最后一个跑完的操作」之间，而且每段只抽固定个数。哪一类崩溃点因此永远抽不到？`assert_every_crash_injection_path_was_exercised` 要求四种写的落点都抽到过、崩溃点要落进回退与抬 F——这几条够不够说明「抽样没把整类窗口漏掉」？举得出一类在真机上会发生、而这套抽法永远不覆盖的崩溃窗口吗？

**K2（判读取哪条根）**：`observed_read_back_after_a_crash` 取 `effective_root`。构造一个状态，让「施加之后实际走的那条根」与「该被判的那一版」不是同一个——例如记录前缀施加到一半、或所选根的实例有回退行时前缀被 W 截断（D23（journal 的角色与格式） 已定项 14 第五条）。那种状态下这条判读会说什么？会不会把一次真违例判成绿？

**K3（模型目录会不会漏版本）**：目录是「每一步之后把根环里的根并进来」攒的，`committed_versions` 的注释说根环 R × S = 24 个槽、一步至多写出四条根，所以下一步之前必然还在环里。这个论证对吗？一步里最多能写出几条根——写行 + 暖机 + 抬 F 的空发布 + 回退那次发布，有没有哪条路能在一步里写出超过 24 条根、把更早的版本挤出环？挤出去之后崩溃点落在那一版上会怎么判？

**K4（截断粒度）**：截断按录制流里的「一次写」为单位。真设备上一次写可能只落一半（撕裂）、也可能乱序落盘。这套截断模型与层 0（门禁 54 号）用的是不是同一套？两处对「崩溃点」的定义有没有差别？差别会让哪一类问题在这一件里看不见？

**K5（并行与确定性）**：`run_crash_injection_campaign` 按种子切片并行。`one_thread_and_four_threads_render_the_same_report` 只比了报告文本。报告里哪些东西是按片次序合并的、哪些是按先到先得攒的？举一个在 32 线程下会与 1 线程不同、而这条用例看不出来的量。

**K6（已知红那一格）**：崩溃状态上「已知红」按清单只记不停。清单里今天只有两条（增补 2 收口表第 ② 行与第 43 行）。一个真的新问题若恰好长成已知红第 0 条那一形（根环转过一整圈时 checker 判 I-3.1 红），会不会被这条只记不停的路吃掉？判别力怎么证？

## 四、分工

| 腿 | 攻击面 |
|---|---|
| 云端攻方（Opus） | K1、K4：抽样罩不住的窗口、截断模型与层 0 的差别。要举得出具体的崩溃窗口或状态，能写成一段可跑的构造更好 |
| 云端正推（Sonnet） | K2、K3：逐条核「判读取 `effective_root`」与「模型目录不漏版本」这两条论证成不成立，引条款原文与代码行号 |
| 本地攻方（英文） | K5、K6：并行合并的确定性、已知红那一格的判别力。按表格逐格填，不许只答 yes / no |

判决写 `research/prompts/m2-supp3-item3-code-r1-main-verification.md`，按路径点名这一轮改过的每个 `crates/*/src/*.rs`（门禁 56 号判形式）。
