# E200 跑前登记：修订6近似树重跑

写于 2026-11-02，装置写之前。
批次 `records/2026-10-31-删兼容旧版本代码.md#17`。不写代码、不跑；登记写死之后由执行员写装置。

<!-- doc-lint:not-numbers Q1 Q2 S1 S2 S8 S9 S10 S12 S14 S15 S16 S17 Z1 Z5 Z7 F1 F2 F9 H3 H15 H16 A1 A8 A12 T1 T2 T3 T4 T5 S40 S41 S42 S43 S44 S45 Z40 Z41 Z42 Z43 F40 F41 F42 K40 K41 K42 K43 K44 A40 A41 A42 A43 A44 A45 A46 M40 M41 M42 M43 M44 M45 M46 M47 M48 M49 M50 M51 M52 P40 P41 P42 P43 P44 P45 P46 -->

## 一、问题

英文名：revision_6_segment_one_on_an_approximate_tree

**为什么取新号、不作 E171 的第 3 次重跑**：E171 的重跑登记（`e171-r<n>-prereg.md`）是同一份输入、同一棵树或照登记拍的树上的同臂重跑，由准入判输入变没变；这里被跑的是另一份实现——修订 6 树（快照摘要 `7726cdd6…`）造不回来，近似树是另拼出来的一棵（`research/prompts/e171-r2-prereg.md` 修订 5 第 5.1 条「拿近似的树重跑，判的是另一份实现，是新实验」；`.claude/singlefs-ai-sop/rules/test-discipline.md`「实验开跑之前，答案不许已经存在」一节「改了就是新实验，重跑」）。用户 2026-11-02 定「照近似树重跑，另立新实验」（`records/2026-10-31-删兼容旧版本代码.md` 第 17 批第 2 条）；派发提示写明它不按准入登记判。所以取 `claim-experiment.sh --next` 给的 E200。

主 agent 给的问题（派发提示「关的是哪几条」原文逐字）：

> 1. 用户 2026-11-02 定「照近似树重跑，另立新实验」：修订 6 那一趟用的三版装置（67e7a549、dbef78ed、b1457bc6）当年没提交，找不回（主 agent 现查，读数在 `records/2026-10-31-删兼容旧版本代码.md` 第 17 批）；修订 6 树（`7726cdd6…`）当年记的 13 份文件 sha256 里 9 份在 git 历史里一版都对不上（执行员量的，`/home/fy5090/.local/share/singlefs/work/compat-removal-e171-r2-run-f/result.md`「结论」第 1 条）。写这一个新实验的跑前登记（取新实验号还是作 E171 的一次重跑，你照编号规矩判，写明依据）：近似树怎么造（取哪个提交、哪几份文件取哪一版、取不到的怎么办）、各臂、对照、判据、失败条款、作废条款，照问题单 Q1、Q2。验收：登记写进 `research/prompts/`，照规矩跑过登记的形式检查；岔路表每行写出够判要量什么。
> 2. 登记里写明与 E171 第 2 次重跑的关系：近似树的读数不进第 2 次重跑任何判据、不替修订 6 那一趟的读数（第 8.5 节第 3 条「谁也不进谁的判据」照用）。验收：登记里有这一句与它压着的条款原文。
> 3. 推一个挂钟与资源量（推的写推的、写依据），重型那几步标重型；跑法照「线程由服务定」，不写线程、内存与台数。

问题单 `research/prompts/e171-revision-6-approximate-tree-questions.md`（sha256 `9a4ff2c5132e8c1f569a9c6b7eac6d0d127f101dc364a5ea1367af3d5d7dcac9`，12 行）第 5–8 行逐字（`sed -n '5,8p'` 原样），就是本登记的岔路单；第 10、12 行（起因、不问）逐字附在表后：

| 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| Q1：今天能从 git 造出的、离修订 6 树（快照摘要 `7726cdd6…`）最近的那棵树，与修订 6 树差在哪几份文件 | 列出的差别只在装置与研究一侧；差别伸进 `crates/` 的产品代码或池级 checker | 近似树与修订 6 那一趟留存产物里记的 13 份文件 sha256 逐份比，对不上的那几份落在哪个包 | 13 份逐份比完、每份写出近似树取的是哪一版（提交号）与它的 sha256 | 开着 |
| Q2：在近似树上重跑修订 6 那一趟第一段之后，E171 第 2 次重跑登记第六节「现状·逐格」里 r1 第一段那一半的对照 ②③（e171-r1 登记第 8.5 节第 2 条：第 1、6、7、10 步之后那几份对照产物，两棵树的逐文件差表）出不出得来、与修订 8 那一趟并排差在哪几格 | 出得来、差的格都落在 Q1 列的差别能解释的地方；出得来、有差的格 Q1 列的差别解释不了；出不来 | 近似树上那一趟各步产物与修订 8 那一趟逐格并排时，差的格的位置与 Q1 差表的对应 | 近似树那一趟第一段第 0–13 步跑完、对照产物两份都写出、逐格并排完 | 开着 |

> 起因：`records/2026-10-31-删兼容旧版本代码.md` 第 17 批——E171 第 2 次重跑要出 r1 第一段对照 ②③，需要修订 6 树与它的导出，而修订 6 那一趟用的三版装置（67e7a549、dbef78ed、b1457bc6）当年没提交过；全部提交与 reflog、stash、悬空对象、第二台与本机整盘都没找到（主 agent 现查，读数在那一批的记录）。用户 2026-11-02 定「照近似树重跑，另立新实验」；E171 第 2 次重跑那一段的 ②③ 照修订 6 记「造不回来」。
> 不问：近似树的读数能不能替修订 6 那一趟的读数（不能：两份实现不同，登记第 8.5 节第 3 条「谁也不进谁的判据」照用）；近似树上谁对谁错（推论，要用另走三方）。

### 读法写死

1. **修订 6 树**：E171 第 1 次重跑修订 6 第 6.3 节拍的那一棵，`row_driver.snapshot_digest`（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/row_driver.py` 第 241 行：每个 `.rs` 与 `Cargo.*` 文件的（相对路径, sha256）排序后再算一个总摘要）为 `7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4`、284 份。它等于第一段新树（`4d071c126a48…` 那一棵，下写「新树」）只换装置文件（修订 6 第 6.3 节拍法；那一趟第 0 步产物 `…-2026-10-05-s1-frozen-tree-r6-r3.out` 的 `key=file_difference` 两行与 `key=device_sha256` 四行由执行员读出、抄进第十二节）。新树拍自 `git rev-parse HEAD` = `31e47115471ba5502a4531d25dffcc921be36899` 加 94 行 `git status --porcelain` 列的未提交改动（新树产物 `…-2026-10-05-s1-frozen-tree.out` 的 `key=git_status` 行）。
2. **「13 份文件 sha256」**：新树产物 `…-2026-10-05-s1-frozen-tree.out` 里 13 行 `key=file_sha256`。其中 12 行（装置之外）在修订 6 树里不变，目标值照抄那一行；装置那一行在修订 6 树里换成了 `67e7a5498425ec3f27379ba8f0e5cd666faaeaac0817006d6938956d6419a156`，Q1 的目标取它，`b1457bc6…`（新树那一份）另报一列、不进判定。同一份 Q1 里另比三样钉值（都出自修订 6 那一趟，都只进 Q1 那一行）：`…-frozen-tree-r6-r3.out` 的 `key=file_sha256` 两行的 `tree=`（`crates/singlefs-core/src/allocator.rs` `61da68ec…`、`crates/singlefs-harness/tests/common/mod.rs` `d2f84876…`）；研究一侧 `revision_8_research_side_of_the_revision_6_run.json` 的 `files` 十一份；整棵树的快照摘要与份数。
3. **Q1 的「差别落在哪」按包分四类**：装置（`crates/singlefs-checker-tier/src/bin/` 下 E171、E170 两个装置文件）；研究一侧（`scripts/experiment-apparatus/` 下）；产品代码与池级 checker（`crates/singlefs-core/`、`crates/singlefs-format/`、`crates/singlefs-checker/`）；别的验证包（`crates/singlefs-harness/`、`crates/singlefs-checker-tier/` 的非 `bin/` 部分、`crates/singlefs-reference-filesystem/`、`Cargo.lock`、各 `Cargo.toml`）。判法只看第 2 条那 13 份加两份 `tree=` 钉值里 sha256 不等的：全部落在前两类 → 候选「只在装置与研究一侧」；有一份落在第三类 → 候选「伸进产品代码或池级 checker」；没有落在第三类、而有落在第四类的 → 问题单两个候选都不是，记「第三种：伸进别的验证包」，照实报、交主 agent（见本节末「未定项」）。没钉 sha256 的那些文件同不同不知道，另报份数（第六节 Q1-未钉），不进这一判。
4. **近似树怎么造（写死，下写「近似树」）**：
   - T1 路径集：`31e47115…` 树里 `Cargo.toml`、`Cargo.lock`、`crates/`（不进 `target/`）下名字以 `.rs` 结尾或以 `Cargo.` 起头的 270 份，并上 94 行里 `A_`、`AM`、`??` 起头的 14 份 `.rs`，共 284 份（本设计员现算，命令在第十三节）；另带 `.cargo/` 下全部与 `crates/mutations.tsv`（不进快照摘要，进逐文件表）。份数不是 284 走停机 S40。
   - T2 不在 94 行里的路径：取 `31e47115…` 那一版，记 `exact_head`（新树就是这一版，按构造相同）。
   - T3 在 94 行里、有钉值的路径（第 2 条那 15 份里落在 94 行的）：候选 blob 是 `git log --all`（含 stash 与 `git reflog --all` 点到的提交）里这条路径的每一版，加 `git fsck --unreachable --no-reflogs` 列出的每个悬空 blob；按内容 sha256 找等于钉值的那一个，找到记 `exact_pinned`（写 blob 号与出处提交，悬空的写 `unreachable`）。
   - T4 装置文件：T3 找不到时，取 `ab35105f…`（提交 `f8a9ddaab`），删掉它对修订 6 树装置那一块差的七行（E171 第 1 次重跑修订 8 第 8.11 节第 4 条：两份的 `diff -u` 只一块 `@@ -3475,6 +3475,13 @@`，第 29 批四行与第 10g 批三行相邻并成一块，即 `ab35105f…` 的第 3478–3484 行；删之前核这七行里含 `SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`）。删完 sha256 等于 `67e7a549…` 记 `exact_reconstructed`，不等记 `approximate_device`、照用。E170 导出装置照 T2、T3、T5 取，不另造。
   - T5 在 94 行里、没有钉值或 T3 找不到的路径：取 `31e47115…` 之后第一次改它的那一版——在 `31e47115…` 的后代提交里，按提交时刻取最早一个、这条路径的 blob 与 `31e47115…` 那一版不同的提交，记 `approximate_first_commit`（写提交号）；新文件（`A_`、`AM`、`??`）取最早加它的那一个提交。后代里一次都没改过（或从没加过）的，取 `31e47115…` 那一版、记 `approximate_head`（新文件记 `missing`，走停机 S40）。
   - 研究一侧（第 2 条那十一份）同 T3 的找法；找不到的取今天主工作区那一份、记 `approximate_today`。修订 6 那一趟研究一侧另有几份没记 sha256（`segment_one_comparator.py`，与已经不在的 `s15_compare.py`），取今天的、记 `unrecorded`。
   - 坏实现两张表取 `04ea50f5…`、`f1bd082e…` 那两版（研究一侧 JSON 的 `files`，按 T3 在 git 里找；今天主工作区里改名改内容的那两份不用）。
   - 不在树里换任何一行、不打补丁、不为编过而改：编不过走停机 S41。
5. **「修订 6 那一趟第一段」**：E171 第 1 次重跑修订 6 第 6.4 节步表的第 0–13 步，各步判法、阈值、限时公式照修订 6、修订 2 第 2.5 节与修订 5 第 5.6 节；跑法照修订 8 第 8.4 节（按单元、第 2b 步现导 E170 坏状态、续跑），用今天研究一侧的修订 11 单元编排（`segment_one_units_revision_11.py`、`segment_one_unit_revision_11.sh`、`run_segment_one_revision_11_multi_host.sh`），树换成近似树。不设的：E171 的 S14（没有 Q5 树）、S15 ①②③（新树、Q5 树都不在）、S15 ④⑤⑥（被比的旧导出都不在，比法改进对照，第 7 条）、S16（树不从仓里拍）；复用层（修订 11）只认这一趟自己产出的键，修订 6 那一趟、E171 第 2 次重跑的键一条都不导入。第 0 步判法另写（第五节），本地常量用近似树里装置自带的那一份，S1 照原文回比、不重取。
6. **「修订 8 那一趟」**：E171 第 2 次重跑现状整表段的第一段（`research/prompts/e171-r2-prereg.md` 第五节 5.1「真实基线」：「r2 那一趟替修订 8 那一趟的位置」），它的树是那一次拍树宣布那一刻的仓，下写「现状树」。今天它还没跑（r2 登记修订 5 标题「现状整表段一份产物都没跑」）。
7. **两份对照产物**（问题单 Q2 够判条件「对照产物两份都写出」）：
   - **对照·修订 6**：近似那一趟对修订 6 那一趟留存在 git 里的产物（`git show 04ceaa904^:research/results/…`）。比三样：第 1、6、7、10 步各产物的 `E7RESULT` 行（去耗时字段照 E171 作废 Z7 的去法；导出根、日志目录、产物后缀与 `-r6-r3` 这类趟名换成同一个占位再比）；两趟比对器产物逐格（`contrast_revision_6_against_revision_8.py` 第 ① 样的身份字段配法）；第 0 步的 `name=frozen_tree` 各钉值（第 2 条）。
   - **对照·修订 8**：E171 第 1 次重跑修订 8 第 8.5 节第 2 条的 ② ③，近似那一趟站在「修订 6 那一趟」的位置：② = 第 1、6、7、10 步之后那几样比，比法照修订 6 第 6.6 节 S15 ④⑤⑥ 的触发观测原文（导出逐文件、逐行），用在近似那一趟与修订 8 那一趟之间；③ = 近似树对现状树的逐文件差表（同第 0 步逐文件表的收法：`Cargo.*`、`.rs`、`.cargo/` 下全部、`crates/mutations.tsv`，不进 `target`）。
8. **Q2 的归因**（候选「Q1 列的差别能解释」怎么判）：对照·修订 8 的 ② 里，近似那一趟与修订 8 那一趟不同的每一格，按它所在的步 k 归一类：对照·修订 6 里第 k 步的 `E7RESULT` 行近似那一趟与修订 6 那一趟逐字相同 →「Q1 解释不了」（这一步上近似树复现了修订 6 那一趟，差的是修订 6 对修订 8，不是近似带进来的）；第 k 步有一行不同、且 Q1 列出至少一份不等或 T4、T5 记了近似 →「Q1 能解释」；第 k 步有一行不同、而 Q1 一份不等都没有、T4、T5 也一份近似都没记 → 走停机 S43。③ 的每一行（文件）照它在近似树里的记号归：`exact_*` 那一份差的是修订 6 对现状，`approximate_*` 那一份差的可能是近似带进来的，只描述、不进 Q2 的判定。Q2 判定只看 ②：差的格全是「Q1 能解释」→ 候选一；有一格「Q1 解释不了」→ 候选二；② 写不出 → 候选三，另写是哪一种写不出（近似树造不成或编不过；近似那一趟停在哪一步；修订 8 那一趟还没跑或导出不在）。
9. **与 E171 第 2 次重跑的关系**：近似树的读数不进 E171 第 2 次重跑的任何判据，不替修订 6 那一趟的读数，也不触发 E171 的任何条款；E171 第 2 次重跑第六节「现状·逐格」r1 第一段那一半的 ②③ 照用户定的记「修订 6 树与导出造不回来」，不拿这里的产物去填。压着的条款是 E171 第 1 次重跑修订 8 第 8.5 节第 3 条，原文：

> 3. **谁也不进谁的判据**。对照产物里任何一格不改任何一趟的判定、不触发任何条款。两趟判得不同的那一行，问题单那一行把两个判定并列写、各带树名（快照摘要），不合成一个，也不写成「最终主线推翻了第 16 批那一版」：差别出自哪一次合入是推论，要用就另走三方。

   照用到这里：近似那一趟、修订 6 那一趟、修订 8 那一趟三趟之间，谁也不进谁的判据；三趟判得不同的格并列写、各带树名，差别出自近似还是出自合入是推论，要用另走三方（问题单「不问」一段）。本实验自己的判定只有问题单 Q1、Q2 两行。

未定项：
- 导出盘量：修订 6 那一趟导出根的大小没有记录，近似那一趟第 1、6、7、10 步的导出要留到修订 8 那一趟出完 ② 才删，要占多少盘推不出。缺：近似那一趟第 1 步导出落盘之后 `du -sh` 一次，按步外推；谁定：主 agent 定放哪、留多久。
- 第 3 条的「第三种：伸进别的验证包」不在问题单的候选里：读数落在这一种时怎么交，由主 agent 定（不改判据，只定报告方式）。

## 二、被测条款与它引的定义

被测条款是 E171 自己的两份登记，不是 kb：E171 第 1 次重跑登记修订 8 第 8.5 节「两趟之间的对照」全节（② ③ 的定义与第 3 条「谁也不进谁的判据」）；E171 第 2 次重跑登记第六节「现状·逐格」那一行、第十一节比对对象表第 5 行（r1 第一段那一半）与修订 5 第 5.1 条（② 的另一边造不回来）。用 `scripts/kb/quote-kb.py` 整段抄进草稿目录 `quotes.md`，回读逐字节一致，再 `>>` 追加在下面（命令与追加后的核对在第十三节）。修订 6 第 6.4、6.6 节（步表、S15 ④⑤⑥ 的触发观测原文）与修订 8 第 8.4 节（跑法）是第 5、7 条读法要照用的定义，太长不抄，执行员照 `research/prompts/e171-r1-prereg.md`（sha256 `5d8fb709e32e0b61df009eb5fd71d2f01285e4eb8bf5ba3dca07d50a141ebf21`）第 1220–1294 行与第 1522–1575 行原文用。

**出处 `research/prompts/e171-r1-prereg.md:1576-1592`（整段抄，未转述）**

```markdown
#### 8.5 两趟之间的对照

两趟：「修订 6 那一趟」是修订 6 树（`7726cdd6…`），第 0–10 步后缀 `-r6-r3`，第 11–13 步后缀 `-r6-r3-resume2`（「主 agent 认定：多机跑法交回」第 5 条）；「修订 8 那一趟」是修订 8 树（H3），后缀 `-r8`（H16）。

照读法 11（「s4 读数只当对照」）的形态：
1. **两趟各自出判定**。各用自己那一趟的比对器（`segment_one_comparator.py`，接第一段的执行员写）只读自己那一趟的产物前缀与导出根，照 Q1–Q5 写判定行。比对器不读另一趟的任何产物：输入路径只认自己那一趟的前缀与导出根，给到另一个前缀就退出（比对器单测钉一条）。
2. **两份判定都写完之后另出对照**。一份新产物 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-<日期>-s1-contrast-revision-6-against-revision-8.out`，每行带 `role=contrast`，由另一份对照脚本写（比对器不引它），逐格并排：
   - Q1：七行 × 三条流 × O1 三个子列、O2、O3 严格与暂代，两趟各自的红态数（O1-对拍是红的调用数）与适用数、峰值、为正段数、期末值，Q1-甲、乙的判定；
   - Q2：C672 七行每次调用的交回值、内存对象同不同与 O1-对拍判定；
   - Q3：第 125 行的 c、截断之后判了的态数、触达否；
   - Q4：三趟 × 三条流 × 两批的（i）（ii）态数、第二截判定、挂载结局；
   - Q5：两个取样点上的 L、RefFS 红、模型崩溃判官红、两者都绿、弃权、`judge_panic` 各态数；
   - 第 1、6、7、10 步之后那几份对照产物（S15 ④⑤⑥ 的比法，用在两趟之间，只描述）；
   - 两棵树的逐文件差表（第 8.3 节）；两趟各行装置那一步的耗时，按台分开。
3. **谁也不进谁的判据**。对照产物里任何一格不改任何一趟的判定、不触发任何条款。两趟判得不同的那一行，问题单那一行把两个判定并列写、各带树名（快照摘要），不合成一个，也不写成「最终主线推翻了第 16 批那一版」：差别出自哪一次合入是推论，要用就另走三方。
4. 问题单 Q1–Q5 的状态由主 agent 照两趟各自的够判逐行写；一趟够判、另一趟还差的，照实写哪一趟还差什么。

```

**出处 `research/prompts/e171-r2-prereg.md:198-198`（整段抄，未转述）**

```markdown
| 现状·逐格 | 现状整表段：上一次登记修订 8 第 8.5 节、修订 10 第 10.5 节那两份对照产物的每一格，r1 修订 6 那一趟对 r2 那一趟；每一格不同的，列出差出来的行，每行归一类：被撤（R）、改引（乙）、树变了（其余） | 描述量；不并进 Q1-撤项的判定；差只来自 R 的格另标 | Q1 够判条件「与 r1 的结论逐格对比」 |
```

**出处 `research/prompts/e171-r2-prereg.md:299-299`（整段抄，未转述）**

```markdown
| 5 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out` | `40ee800a8681caf503c05131fbd05fcdba7a9514a38178660c55195330b1cabc` | 811 | `replay.sh` 第 1827 行（`driver_e171_r1_s1_revision_6_comparator`）；现状整表段照上一次登记修订 8 第 8.5 节的对照脚本比 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-s1-comparator.out` | 现状·逐格（r1 第一段那一半） |
```

**出处 `research/prompts/e171-r2-prereg.md:414-418`（整段抄，未转述）**

```markdown
5.1 **第一段对照 ② 要的修订 6 导出重出不来（量的，2026-11-02）**：
   - 修订 6 树（快照摘要 `7726cdd6…`，284 份）是第一段新树（`4d071c12…`）只换装置拍成的（上一次登记修订 6 第 6.3 节）；第一段新树拍自 HEAD `31e47115…` 加未提交的改动（`git show 04ceaa904^:research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree.out` 记 94 行 `git_status`，其中 `.rs` 与 `Cargo.*` 约九十份）。
   - 那份产物记了 13 份文件的 sha256：拿 `git log --all -- <路径>` 的每一版（每份至多 60 版）现算比，有 9 份一版都对不上（`model.rs`、`model_comparison.rs`、`model_compares_inode_creations_across_two_inode_leaf_containers.rs`、`mount_transparency.rs`、`crash_injection.rs`、`crash_judge_gpu.rs`、`singlefs-checker` 的 `image.rs` 与 `walk.rs`、装置 `b1457bc6…`）。装置路径在 git 里只有 10 版，`67e7a549…`（修订 6 树装置）、`dbef78ed…`、`b1457bc6…` 都不在，只有 `ab35105f…` 一版（提交 `f8a9ddaab`）；P6 补丁与那几份 fixture 在 git 里也没有。
   - 所以修订 6 树逐字节造不回来，从它编的两个二进制与第 1、6、7、10 步的导出也重出不来；拿近似的树重跑，判的是另一份实现，是新实验。对照脚本 ③（两棵树的逐文件差表）同样要修订 6 树；①、④ 只读两趟的产物，修订 6 那一趟的产物在 git 里取得到。
   - `s15_compare.py` 照登记重写入库那一半没写：它只给 ② 用，② 的另一边造不回来。第六节「现状·逐格」r1 第一段那一半照现在的写法出不来；改成只出 ①④、②③ 记缺，是第六节那一行的报告方式，交主 agent 问用户。
```

## 三、实现今天的样子

2026-11-02 现查（主工作区，`git rev-parse HEAD` 为 `59ce5a86550de2c2c9bc6b51b34b4f881313682c`）。被跑的不是今天的 `crates/`，是第一节读法 4 造出的近似树；今天的 `crates/` 只在现状树那一侧出现。
- **E171 装置今天**：`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 12 960 行，sha256 `cbecfcebadf9f9c6135230071625dd5477da7b92942e2795ec693936318739bc`（E171 第 2 次重跑登记第三节记的是 `22400a1b…`，之后又改过）；`crash_amplification::` 与 `crash_state_enumeration::` 命中 7 处。git 里这条路径唯一一份修订 6 前后的版本是 `ab35105f…`（提交 `f8a9ddaab`，12 645 行），它的这两个前缀命中 0 处。
- **`31e47115…` 那一版的 `crates/singlefs-checker-tier/Cargo.toml`**：第 27 行有 `gpu` 特性；近似树里的装置不走崩溃判段（上一条 0 处），编、跑都不带它。
- **快照摘要**：`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/row_driver.py` 第 241 行 `snapshot_digest`（只收 `.rs` 与 `Cargo.*`）。
- **对照脚本**：同目录 `contrast_revision_6_against_revision_8.py`（388 行）第 54–55 行 `S15_COMPARE` 钉到修订 6 草稿 `e171-r1-segment1-rev6/s15_compare.py`（sha256 `346fcc2e…`），那份文件与草稿目录都不在（E171 第 2 次重跑修订 5 第 5.1 条），所以它的 ② 起不来；它的 ① 的身份字段配法在第 58 行 `IDENTITY_FIELDS`，对照·修订 6 照这一份配。
- **第 0 步判法**：同目录 `frozen_tree_lines_revision_8.py`（995 行）判的是 S16（树对仓逐文件差为空），近似树不从仓里拍，用不上；这一次另写。
- **单元编排**：同目录 `segment_one_units_revision_11.py`（556 行；第 60–61 行按 sha256 载入修订 8 那一份）、`segment_one_unit_revision_11.sh`（458 行）、`run_segment_one_revision_11_multi_host.sh`。
- **修订 6 那一趟的研究一侧**：同目录 `revision_8_research_side_of_the_revision_6_run.json` 的 `files` 十一份；其中 `rows_orchestrator.py`（钉 `7bfbdb84…`）在 git 里只有 `e6f3a370…` 一版、`run_segment_one_on_the_revision_6_tree_multi_host.sh`（修订 6 那一趟用的 `ec89ec78…`）在 git 里只有 `732473a1…` 一版（本设计员现算，第十三节）。
- **`crates/` 里没有「近似树」这一样东西**：`grep -rn 'approximate_tree\|approximate tree' crates/` 0 处；造树、Q1 比对、Q2 归因都是研究一侧的新脚本（第五节）。

## 四、跑之前已经存在的数

设计时读到或自己算出的，照实列，不删。
1. **执行员 run-f 量的**（`/home/fy5090/.local/share/singlefs/work/compat-removal-e171-r2-run-f/result.md`「结论」第 1 条与 E171 第 2 次重跑修订 5 第 5.1 条）：13 份里 9 份拿 `git log --all -- <路径>` 每一版（每份至多 60 版）现算都对不上：`model.rs`、`model_comparison.rs`、`model_compares_inode_creations_across_two_inode_leaf_containers.rs`、`mount_transparency.rs`、`crash_injection.rs`、`crash_judge_gpu.rs`、`singlefs-checker` 的 `image.rs` 与 `walk.rs`、装置 `b1457bc6…`；装置路径 git 里 10 版只有 `ab35105f…`。影响：Q1 在「提交历史」这一个候选源上的答案跑之前已有一半；这是写 T3（多找悬空 blob 与 reflog）、T4（从 `ab35105f…` 删七行）的起因。Q1 的判法（第一节读法 3）出自问题单候选的字面，不从这里来；这里没比的那 2 份 `tree=` 钉值与十一份研究一侧，设计时不知道同不同。
2. **主 agent 现查的**（`records/2026-10-31-删兼容旧版本代码.md` 第 17 批第 2 条）：全部提交、reflog、3 个 stash 里装置 15 个不同 sha256，没有 `67e7a549`、`dbef78ed`、`b1457bc6`；13 300 个悬空 blob 里大于 200K 的 3 700 个 0 命中；两台整盘同名文件 0 命中。影响：T3 在装置上大概率找不到，T4 就是为这一种写的；别的 8 份在悬空 blob 里找没人做过。
3. **E171 第 1 次重跑修订 8 第 8.11 节第 4 条**：`ab35105f…` 对 `67e7a549…` 的 `diff -u` 只一块 `@@ -3475,6 +3475,13 @@`。影响：T4 的删法；删完等不等于 `67e7a549…` 本设计员没算（那是 Q1 的读数）。
4. **本设计员读的两份冻结树产物**（只为定第一节读法 1、2 的目标值；`git show 04ceaa904^:` 取到草稿目录）：`…-2026-10-05-s1-frozen-tree.out`（125 行，sha256 `b720e5d28dcaead6e7c7262fd82185856e869385dad30674bf9689cb658f24c6`）的键种、94 行 `git_status`、13 行 `file_sha256` 的路径与值、`snapshot_digest`；`…-2026-10-05-s1-frozen-tree-r6-r3.out`（62 行，sha256 `786e48010de9622d623198da5eabecba9694afd3ca73bf71cee2e3f241ca4b90`）的键种、`device_sha256` 四行、`file_sha256` 两行、`binary_sha256` 两行、三个快照摘要与 `p6` 一行。影响：都是目标值，不是判据门槛；没拿它们去 git 里比。
5. **本设计员现算的**：T1 路径集 270 + 14 = 284，与快照摘要的份数相同（停机 S40 在设计时不触发）；94 行按状态 `M_` 73、`A_` 11、`_M` 4、`MM` 3、`AM` 2、`??` 1；其中 `.rs` 与 `Cargo.*` 89 份；按目录 `singlefs-harness` 54、`singlefs-checker-tier` 29、`singlefs-core` 7、`singlefs-checker` 2、`Cargo.lock` 1、`crates/mutations.tsv` 1；`31e47115…` 是 HEAD 的祖先；研究一侧两份跑法文件在 git 里各只一版、都不是修订 6 那一趟钉的那一版（第三节）。影响：T5 要处理的路径有 89 份上下（减去 T3 找到的）；研究一侧至少两份记 `approximate_today`。
6. **E171 第 2 次重跑登记第四节第 1 条**（读那份登记时读到的 E171 实验页标题，含第 1 次重跑第一段 Q1–Q5 的判定词）。影响：E200 不判 E171 的 Q1–Q5，那几个判定词不进任何门槛。
7. **挂钟依据**：E171 第 1 次重跑修订 6 第 6.4 节步表（`process_wall seconds=14581.558`、流三每态 0.82 秒带探针、第 3 步 18 210 秒、带流三两项 9 698 与 8 128 秒）与修订 8 第 8.9 节（合计约 14–16 小时，本机约 9.5、第二台约 7.0 小时）。影响：只进第十四节。
8. **没打开的**：修订 6 那一趟第 1–13 步的任何产物与比对器产物（第 4 条两份冻结树产物之外），E171 实验页正文，`research/results/` 下别的文件。

## 五、臂、阳性对照、真实基线

### 5.1 臂（问题单的候选；每一边写成支持它的人认的样子）

| 行 | 一边 | 另一边 |
|---|---|---|
| Q1 | **只在装置与研究一侧**。支持它的人会要求：找版本不只翻提交历史，悬空 blob、reflog、stash 一起找（T3）；装置对的是修订 6 树那一份 `67e7a549…`，不是新树的 `b1457bc6…`，且先试 T4 的复原；研究一侧照算进「差别」，不算进产品代码 | **伸进产品代码或池级 checker**。支持它的人会要求：只认内容 sha256 逐字节相等，不做任何去空白、去注释的归一；`…-r6-r3.out` 那两份 `tree=` 钉值（`allocator.rs` 在 `singlefs-core`）一起比；没钉 sha256、按 T5 近似的文件照列份数与所在包，不藏；第三种（只伸进别的验证包）不并进候选一 |
| Q2 | **出得来、差的格都由 Q1 解释**。支持它的人会要求：归因按步、按对照·修订 6 那一步的产物逐字相同不相同判（第一节读法 8），不按「看起来像」；归不了的格单列，不算进候选二 | **出得来、有格 Q1 解释不了**。支持它的人会要求：② 的比法逐字照修订 6 第 6.6 节 S15 ④⑤⑥ 的触发观测原文重写，不另立比法；对照·修订 6 去掉的只有耗时字段与路径、趟名这几样占位，别的字段一个不去 |
| Q2（第三个候选） | **出不来**。支持它的人会要求：写不出的原因分三种各报（近似树造不成或编不过；近似那一趟停在哪一步、哪条停机或作废；修订 8 那一趟没跑或导出不在），不合成一个「出不来」 | 同上两格 |

### 5.2 阳性对照（每一条臂、每一个新判法都跑；单测里手造输入，不碰真产物）

| # | 对照 | 怎么造 | 必须看到 |
|---|---|---|---|
| P40 | 造树（T2–T5 各一格） | 临时 git 仓：一份钉值路径的那一版只作悬空 blob 在；一份钉值路径哪儿都没有；一份没钉值路径在两个后代提交里各改一次；一份新文件在后代里加两次；一份后代里没改过 | 依次记 `exact_pinned`（`unreachable`）、落到 T5、取较早那一提交、取最早加它的那一提交、`approximate_head` |
| P41 | Q1 判法三种结局 | 手造逐份表：只装置不等；`singlefs-core` 一份不等；只 `singlefs-harness` 一份不等 | 依次判候选一、候选二、第三种 |
| P42 | T4 删七行 | 手造一份文件，第 3478–3484 行是一块带那个枚举名的七行 | 删完 sha256 等于用例另算的期望；七行里没有那个名字时退出、报停机 S44 |
| P43 | ② 的三种比法（S15 ④⑤⑥ 照原文重写，各一格） | 每种比法手造两份导出：逐字节相同一对；④ 在 `states.tsv` 改一个字节；⑤ 在流一某一次调用的内存对象里多一处不同；⑥ 在流三第一处不同提到「写 txg 26 那条根的那次调用」之前 | 相同那一对报 0 处不同；三处改动各报在点名的那一格 |
| P44 | Q2 归因三种 | 手造：第 k 步产物两趟相同、② 有一格不同；第 k 步产物不同、Q1 有一份不等；第 k 步产物不同、Q1 一份不等都没有、T4 T5 一份近似都没记 | 依次判「Q1 解释不了」「Q1 能解释」、报停机 S43 |
| P45 | 对照·修订 6 的归一 | 两行只差耗时字段与 `-r6-r3` 趟名；两行差一个计数字段 | 前一对相同，后一对不同 |
| P46 | E171 自己的阳性对照 | E171 第 1 次重跑第 3、4 步十项，在近似树上照原判法与 A8 精确数跑 | 照 E171 的 F1；不过的那一项走作废 Z41 |

### 5.3 真实基线

- Q1：修订 6 树那几份钉值本身（第一节读法 2）。
- Q2：修订 6 那一趟留存在 git 里的产物（对照·修订 6）；修订 8 那一趟（第一节读法 6，对照·修订 8）。

### 5.4 装置写在哪（写死）

装置走研究一侧 `scripts/experiment-apparatus/e200_revision_6_segment_one_on_an_approximate_tree/`（Python 与 shell，不与 `crates/` 共用代码）。不改 `crates/` 下任何文件；E171 的入库装置（`e171_three_oracle_collusion_after_the_oracle_fixes` 与 `e170_o3_spec_executor_export` 两个 bin）不改一行，只在近似树里那一份上编、跑；本实验没有要读 `crates/` 常量的量，S1 的本地常量用近似树里装置自带的那一份。不改 E171 研究一侧目录里任何一份文件（那里有按 sha256 的钉链，E171 第 2 次重跑在用）：要用的模块按路径载入、开跑先核 sha256；要改的拷进 E200 目录再改，文件头写出处与原 sha256。新写：
- `build_approximate_tree.py`：第一节读法 4 的 T1–T5，产物是近似树与一份逐文件清单（路径、记号、出处提交或 blob、sha256）。
- `approximate_tree_lines.py`：第 0 步判法，替 E171 修订 8 那一份：清单与树逐份相同、快照摘要每次起跑与续跑都等于造树那一刻（作废 Z40）、两个 release 二进制的 sha256、A12 十二项（照 E171 原判法）、Q1 那几行（第六节）。
- `contrast_against_revision_6.py`（对照·修订 6）、`s15_methods_between_runs.py`（② 的三种比法，照 E171 第 1 次重跑修订 6 第 6.6 节 S15 ④⑤⑥ 原文）、`contrast_against_revision_8.py`（② ③）、`attribute_q2.py`（第一节读法 8）。
- 跑法 `run_segment_one_on_the_approximate_tree.sh`：照修订 8 第 8.4 节与修订 11 单元编排，树、第 0 步判法、产物后缀（`-e200`）、导出根换成这一趟的；同一条命令重起就是续跑。
- 单测 `test_build_approximate_tree.py`、`test_contrasts.py`；变异表 `scripts/mutation/tables/e200_revision_6_segment_one_on_an_approximate_tree.tsv`，经 E171 研究一侧的 `mutation_driver.py` 那种做法跑（拷进 E200 目录或按路径载入）。

### 5.5 实现量与分段

**实现量**（推的，没量）：造树约 250 行、第 0 步判法约 200 行、两份对照约 350 行、三种比法约 250 行、归因约 120 行、跑法约 150 行、单测约 500 行、变异 12 行。不另写检查器、触发式重算或崩溃枚举；历史族、几何格、崩溃支线、坏镜像都照 E171 第一段原样，不加。

**分段**（每段交回之后主 agent 对着问题单判续不续；一行开着的都不剩就停，后面的段不跑）：

| 段 | 做什么（按能最先让某一行够判的次序） | 补哪一行 | 补完够不够判 |
|---|---|---|---|
| 第一段 | 写造树与第 0 步判法、单测、变异；主 agent 交造树与 Q1 比对；树里 release 编两个 bin（只编，看编不编得过） | Q1 全部；Q2 的「出不来」里「造不成或编不过」那一种 | Q1 够判（13 份逐份比完）。编不过的，Q2 也够判（候选三，那一种） |
| 第二段 | 写跑法与对照·修订 6；主 agent 交近似树上第一段第 0–13 步；出对照·修订 6 | Q2 的归因依据（各步两趟同不同） | Q2 不够判：还差修订 8 那一趟。近似那一趟停机或作废的，Q2 够判（候选三，那一种） |
| 第三段 | 修订 8 那一趟第一段跑完之后：写 ② ③ 与归因；出对照·修订 8 与归因 | Q2 | 够判 |

第三段的前提是 E171 第 2 次重跑现状整表段的第一段跑完、它第 1、6、7、10 步的导出还在；那一趟什么时候跑不归本实验，主 agent 定。第二段的导出要留到第三段做完。

## 六、报哪些量与各自的判据

每个量各占一行，合取留给人做。问题单只有 Q1、Q2 两行；对不上任何一行的量不登记。E200 不判 E171 的 Q1–Q5：近似那一趟比对器出的那几个判定只进对照·修订 6 当格子比。

| 量 | 怎么算 | 门槛与判定 | 对应问题单哪一行、取什么值翻面 |
|---|---|---|---|
| Q1-逐份 | 第一节读法 2 的 13 份加两份 `tree=` 钉值：每份写路径、所属类（读法 3）、目标 sha256、近似树取的哪一版（记号、提交号或 blob 号）与它的 sha256、相等否 | 读法 3：不等的全在装置与研究一侧（含一份不等都没有）→ 候选一；有一份不等落在产品代码与池级 checker → 候选二；只多出别的验证包 → 第三种 | Q1：`allocator.rs`、`singlefs-checker` 的 `image.rs`、`walk.rs` 三份里有一份不等即翻向候选二 |
| Q1-装置 | T3 找没找到 `67e7a549…`；没找到时 T4 删七行之后的 sha256、等不等于 `67e7a549…`；另报与 `b1457bc6…` 等不等 | 描述（装置落在第一类，不翻候选） | Q1（13 份里装置那一份的「取哪一版」） |
| Q1-研究一侧 | 研究一侧 JSON 十一份逐份：取的哪一版、sha256、相等否；`unrecorded` 的几份另列 | 描述（落在第二类，不翻候选） | Q1（候选一字面里的「研究一侧」） |
| Q1-未钉 | 94 行里不在钉值里的路径：按记号（`exact_pinned`、`approximate_first_commit`、`approximate_head`）与读法 3 的四类数份数，逐份列路径与提交号 | 描述，不进候选的判定；与 Q1-逐份并列交 | Q1（「差别」里看不见的那一半有多大） |
| Q1-全树 | 近似树的快照摘要与份数，对 `7726cdd6…` / 284 | 相等 → 另写「近似树在 `.rs` 与 `Cargo.*` 上就是修订 6 树」；不等 → 照 Q1-逐份 | Q1：相等时 Q1-逐份必全等（不全等走停机 S40，判法自相矛盾） |
| Q2-编 | 近似树里 release 编 E171 与 E170 两个装置 bin 的退出码与诊断末 20 行 | 有一个非 0 → Q2 候选三（造不成或编不过） | Q2：编不过即翻向候选三 |
| Q2-各步 | 近似那一趟第 0–13 步每个单元：绿 / 红、触发的条款（E171 的作废停机原文，与本登记 Z40–Z43、S40–S45） | 第 1、6、7、10 步有一个单元没绿（停机或作废） → Q2 候选三（近似那一趟停在哪一步）；别的步没绿只描述 | Q2：② 要的四步有一步出不来即翻向候选三 |
| 对照·修订 6 | 第一节读法 7：第 1、6、7、10 步各产物 `E7RESULT` 行归一后逐行比、两趟比对器产物逐格比、第 0 步钉值逐项比；每步报同的行数、不同的行数与逐行原文 | 描述；第 1、6、7、10 步的同不同进 Q2-归因 | Q2（归因依据） |
| 对照·修订 8 ② | 第一节读法 7 的 ②：每步、每种比法（S15 ④⑤⑥ 原文）逐格报近似那一趟与修订 8 那一趟同不同 | 描述；不同的格进 Q2-归因 | Q2：一格不同都没有时 Q2 判候选一（没有要解释的差） |
| 对照·修订 8 ③ | 近似树对现状树逐文件差表，每行带近似树里的记号 | 描述，不进 Q2 判定 | Q2（够判条件「两份都写出」的一部分） |
| Q2-归因 | 第一节读法 8：② 每一格不同，归「Q1 解释不了」「Q1 能解释」 | 全是「Q1 能解释」→ 候选一；有一格「Q1 解释不了」→ 候选二 | Q2：有一格解释不了即翻向候选二 |
| 附带（够判后不跑） | 近似那一趟对修订 8 那一趟的比对器逐格（① 那一种）；两趟第 11–13 步各行耗时（④ 那一种） | 照数报 | 不对应任何一行 |

**可跑性与判别力**（第 3a 步）：
- Q1：要算 sha256 的 blob 数 = 89 份路径各自的版本（run-f 每份至多 60 版，至多 5 340 个）+ 悬空 blob 13 300 个（主 agent 数的）≈ 1.9 万个，每个至多约 600 KB（装置 12 645 行），`git cat-file --batch` 读加 sha256，推的 2–5 分钟、内存几十 MB。分辨率：内容 sha256 逐字节，没有噪声。
- Q2：分辨率是归一之后逐字节；E171 装置本身是确定的（E171 作废 Z1 两次基线逐字节比、Z7 主取样点两次逐字节比），这两条在近似那一趟里各自复判，过了才说「不同不是噪声」（失败 F41）。归因的粒度是「步」，不是「格」：② 里一步内哪一格出自近似、哪一格出自修订 6 对修订 8，这一判法分不开，照第一节读法 8 按步记，写进结论的射程句。
- 各步的态数与耗时照 E171 第 1 次重跑修订 6 第 6.4 节：第 7 步带流三、带探针的进程挂钟是量的（旧树 `process_wall seconds=14581.558`）；流三每态 0.82 秒（带探针）是推的、与这个进程挂钟同源，两数相除折约 17782 态（推的，不是数出来的态数）；第 7、9 步各约 4.1 小时；不带探针的带流三单元 8128–9698 秒（第 3 步各产物的修改时刻）。（第十二节修订 1 第 5 条改写，原文照录在那一条）
- 第 3a 步第 ③ 条要触发的事件：流三转过 24 槽根环、从 txg 26 起回收，是 E171 自己的前提（S1 那条根环表与流三段长表在第 0 步回比），本实验不另判；第 1 步只跑流一流二，不碰它。
- 设计时自己算出的预期差：Q1 那 15 份里至少 9 份在提交历史里对不上（第四节第 1 条）；它们在悬空 blob 里在不在、T4 复原等不等，没算。Q2 的差设计时算不出（要跑）。

## 七、钉绝对值的断言

**出自被测条款本身**（不符走「条款可能错」：两边都查，交主 agent，不作废）：

| # | 断言 | 出处 |
|---|---|---|
| K40 | 修订 6 树快照摘要 `7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4`、284 份 | E171 第 1 次重跑修订 8 第 8.5 节「「修订 6 那一趟」是修订 6 树（`7726cdd6…`）」；修订 6 第 6.3 节 |
| K41 | 修订 6 树装置 `67e7a5498425ec3f27379ba8f0e5cd666faaeaac0817006d6938956d6419a156`；`ab35105f861d509457296741d2c60515a3a94b9fefe77944f03d1b358afd45df` 是它加第 29 批四行与第 10g 批三行 | 修订 8 第 8.3 节表「装置四处的 sha256」一行与第 8.1 节「`ab35105f…` 是……」一句 |
| K42 | 新树 HEAD `31e47115471ba5502a4531d25dffcc921be36899`、未提交改动 94 行 | E171 第 2 次重跑修订 5 第 5.1 条 |
| K43 | 修订 6 那一趟产物的趟名：第 0–10 步后缀 `-r6-r3`，第 11–13 步 `-r6-r3-resume2` | 修订 8 第 8.5 节首段 |

**独立算出、用命令核过的**（不符就作废那一格；命令在第十三节）：

| # | 断言 | 怎么算的 |
|---|---|---|
| A40 | T1 路径集：`31e47115…` 里 `.rs` 与 `Cargo.*` 270 份 + 94 行里新加的 `.rs` 14 份 = 284，两组交集 0 | `git ls-tree -r --name-only` 过滤后 `wc -l`；`comm -12` |
| A41 | 94 行按状态 `M_` 73、`A_` 11、`_M` 4、`MM` 3、`AM` 2、`??` 1；`.rs` 与 `Cargo.*` 89 份 | `cut -c1-2 \| sort \| uniq -c`；`grep -c -E` |
| A42 | 新树产物 125 行、sha256 `b720e5d28dcaead6e7c7262fd82185856e869385dad30674bf9689cb658f24c6`；修订 6 那一趟第 0 步产物 62 行、sha256 `786e48010de9622d623198da5eabecba9694afd3ca73bf71cee2e3f241ca4b90`；`key=file_sha256` 各 13 行与 2 行 | `git show 04ceaa904^:… \| sha256sum`、`wc -l`、`grep -c` |
| A43 | `ab35105f…` 在提交 `f8a9ddaab`、12 645 行；`crash_amplification::` 与 `crash_state_enumeration::` 0 处 | `git show f8a9ddaab:<装置> \| sha256sum`、`wc -l`、`grep -c` |
| A44 | 15 份钉值按读法 3 分：装置 1、产品代码与池级 checker 3（`allocator.rs`、`image.rs`、`walk.rs`）、别的验证包 11、研究一侧 0 | 照第四节第 4 条那几行的路径数 |
| A45 | 问题单 sha256 `9a4ff2c5132e8c1f…`（全值在第一节）、12 行 | `sha256sum`、`wc -l` |

每一条互比（近似对修订 6、近似对修订 8）旁边，A40–A44 照样能红：造树与比对一起错了，份数、状态分布、钉值的份数与分类不会跟着错。

## 八、轨迹与几何敏感性

**轨迹**：Q1 没有轮次。Q2 被谓词消费的是「这一格两趟同不同」，也没有轮次；E171 自己被谓词消费的量（逐段红态数的峰值、为正段数、期末值）照 E171 比对器原样出，在对照·修订 6 里逐格比，不另报。

**几何敏感性**（`.claude/rules/mutation-sampling.md` 第六类；旋钮是造树的 T5 取法）：

| 判决 | 主取样点 | 方向相反的取样点 | 怎么报 |
|---|---|---|---|
| Q1 | 近似树甲：T5 取 `31e47115…` 之后第一次改它的那一版 | 近似树乙：T5 一律取 `31e47115…` 那一版（新文件照取最早加它的那一版，否则编不起来） | Q1-逐份与 Q1-全树两棵各报；钉值文件 T3 找不到时两种取法都不等于钉值，所以 Q1 的候选在两点上同判（推的），照实报两点 |
| Q2 的归因依据 | 近似树甲上第 0–13 步 | 近似树乙只编、只跑第 0、1 步（流一流二，约 25 分钟，推的） | 第 1 步对照·修订 6 两棵树各报同不同；判得不同记「不稳定」并列 |

**判别力自证**（`test_contrasts.py`）：把读法 3 里 `crates/singlefs-core/` 挪进别的验证包，P41 第二格必须由候选二转第三种；把读法 8 的「逐字相同」挪成「行数相同」，P44 第二格（造成两趟行数相同、内容不同）必须由「Q1 能解释」转「Q1 解释不了」。

## 九、变异

进新表 `scripts/mutation/tables/e200_revision_6_segment_one_on_an_approximate_tree.tsv`，单测模块 `test_build_approximate_tree`、`test_contrasts`。每条写明在哪个取样点上改变输出：

| # | 改哪 | 改成什么 | 在哪个取样点上改变输出（会红的那一格） |
|---|---|---|---|
| M40 | T3 的候选 blob | 不收悬空 blob | P40 第一格：原式 `exact_pinned`（`unreachable`），变异落到 T5 |
| M41 | T5 取哪一个后代提交 | 取最晚的 | P40 第三格：原式取较早那一提交，变异取较晚那一个 |
| M42 | T5 的新文件 | 取最晚加它的提交 | P40 第四格 |
| M43 | T2 取哪一版 | 取今天 HEAD 那一版，不取 `31e47115…` | P40 另加一格「不在 94 行、后代里改过」：原式取 `31e47115…` 那一版，变异取后代那一版 |
| M44 | 读法 3 的类 | `crates/singlefs-core/` 不算产品代码 | P41 第二格：原式候选二，变异第三种 |
| M45 | Q1-逐份「相等」 | 只比文件长度 | P41 另加一格「钉值文件与目标同长不同内容」：原式不等，变异相等 |
| M46 | T4 删哪几行 | 第 3477–3483 行（差一） | P42 第一格：sha256 与期望不同 |
| M47 | T4 的名字核对 | 不核 | P42 第二格：原式退出报停机 S44，变异照删 |
| M48 | ② 的 ④ 比法 | 不比 `states.tsv` | P43 ④ 那一格：原式报 1 处，变异报 0 |
| M49 | ② 的 ⑥ 比法 | 第一处不同取最后一处 | P43 ⑥ 那一格：报的位置不同 |
| M50 | 读法 8 的归类 | 第 k 步两趟相同记「Q1 能解释」 | P44 第一格：原式「Q1 解释不了」，变异「Q1 能解释」 |
| M51 | 读法 8 的停机 | 不判停机 S43 | P44 第三格：原式报 S43，变异记「Q1 能解释」 |
| M52 | 对照·修订 6 的归一 | 去掉全部数字字段 | P45 后一对：原式不同，变异相同 |

变异全抓才算这几样判法有判别力；没红的照 `.claude/rules/mutation-sampling.md`「没红的三类，判据不同」分；无效的照第八类另列、换一条真改行为的补。E171 的装置与研究一侧不改，不加它们的变异行。

## 十、失败条款

| # | 条款 | 什么观测会让它触发 |
|---|---|---|
| F40 | 近似树上 E171 的阳性对照不过：近似那一趟第 3 或第 4 步那一列照 E171 的 F1 整轮作废；Q2 记候选三（近似那一趟停在第 3 或第 4 步），不拿那一趟的第 1、6、7、10 步去归因 | 第 3、4 步任一项单元红在 E171 F1 点名的那种（A8 精确数不符或该红的没红） |
| F41 | 近似那一趟自己不确定：Q2-归因记「不够判」，不判候选一、二 | 近似那一趟 E171 作废 Z1（第 7、9 步逐字节）或 Z7（第 10 步两次逐字节）判法退 1 |
| F42 | Q1 的钉值不够描述差别：Q1 判候选一，而对照·修订 6 第 1 步（只跑流一流二）就有一行不同。照最弱读法记 Q1 那一判在 Q2 上不成立，并列出 Q1-未钉里第 1 步编得到的那几份（`singlefs-harness` 与 `singlefs-core` 下 T5 近似的） | Q1-逐份判候选一，且对照·修订 6 第 1 步不同的行数 ≥ 1 |

## 十一、作废条款与停机条款

**作废**（那一次的输出不算结果；修好之后重跑，不改判据）：

| # | 条款 | 触发观测 |
|---|---|---|
| Z40 | 跑的不是造出来的那棵近似树（E171 作废 Z5 的这一趟读法） | 某次起跑、续跑时近似树的快照摘要或逐文件清单与造树那一刻记下的不同；或某个单元的二进制 sha256 不是第 0 步记的那一个 |
| Z41 | 本实验自己的判法没证明会红 | P40–P45 单测有一格红，或第九节变异有一条没被抓或无效；这期间出的对照与归因产物作废 |
| Z42 | 对照·修订 6 读的不是修订 6 那一趟的产物 | 执行员第一次从 `git show 04ceaa904^:` 取每份产物时记下 sha256 与行数，之后每次读之前核，不同；或取的产物名不带 `-r6-r3`（或 `-r6-r3-resume2`） |
| Z43 | 对照·修订 8 读的不是修订 8 那一趟 | 读的导出根、产物前缀与 E171 第 2 次重跑那一次拍树宣布行与 H16 记的不同 |

近似那一趟另沿用 E171 第 1 次重跑的作废 Z1–Z7（Z5 读作 Z40）。

**停机**（装置或判法与 `crates/` 对不上、或造出来的东西与读法不合：两边都查，既不作废也不当结果，交主 agent；照 `.claude/rules/implementation-first.md` 第 4 条，不默认装置对，也不默认近似树对）：

| # | 条款 | 触发观测 |
|---|---|---|
| S40 | 近似树的路径集或判法自相矛盾 | T1 份数不是 284；T5 有一份新文件记 `missing`；Q1-全树相等而 Q1-逐份有一份不等 |
| S41 | E171 的本地常量在近似树上不成立 | 第 0 步 `stream-constants --streams 1,2,3` 编不过、退非 0、`holds=false` ≥ 1 行或 `s1_failed_checks` ≠ 0。不重取常量（那是改装置），Q2 记候选三（停在第 0 步） |
| S42 | 第 2 步 E170 导出对不上近似树上的装置 | E171 的 S2 原文触发 |
| S43 | 归因自相矛盾 | 第一节读法 8 末种：第 k 步两趟不同，而 Q1 一份不等都没有、T4 T5 一份近似都没记 |
| S44 | T4 删的不是那一块 | `ab35105f…` 第 3478–3484 行里没有 `SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults` |
| S45 | 近似树上逐行要的东西不齐 | A12 十二项（E171 原判法）不过；或第一段 16 行（E171 `STEP_ROWS` 之并）在近似树的 `crates/mutations.tsv` 里按名字取不到或锚点不是恰 1 次。照 E171 修订 8 第 8.3 节 H9 的办法：执行员起跑前交主 agent 定照不照跑 |

近似那一趟另沿用 E171 第 1 次重跑的停机 S3、S5–S13、S17（「登记的那棵冻结树」读作近似树甲）；E171 的 S1 由 S41 接替，S2 由 S42 接替；S14、S15 ①–⑥、S16 不设（第一节读法 5）。

**够判停机**：每段交回附问题单两行的表（已够判 / 还差什么 / 剩下的量能不能让它翻面）。Q1 在第一段够判；Q2 在第一段（编不过）、第二段（近似那一趟停机或作废）或第三段够判。一行开着的都不剩就停，没跑的量逐条标「够判后未跑」；附带量一律够判后不跑。

**比对对象**：不适用。这不是重跑登记，没有「上一次产物」要逐份比；对照·修订 6 读的那几份产物在第一节读法 7 与作废 Z42 里点名。

## 十二、修订

（留空；装置写之后、产物之前由执行员写，只许收严或补臂。第一节读法 1 要执行员抄的两行 `file_difference` 与四行 `device_sha256`、Z42 记下的各份产物 sha256、近似树甲乙两份逐文件清单的 sha256、修订 8 那一趟的导出根与产物前缀，抄在这里。）

### 修订 1（2026-11-02，执行员第一段）

时点：第一段的装置（`scripts/experiment-apparatus/e200_revision_6_segment_one_on_an_approximate_tree/` 下造树、Q1 与编译那几份）与变异表写完，单测与变异一次没跑，产物一次没跑。下面几条都不依据单测读数（单测还没跑），依据写在每条里；都不动第六、七、九节任何一条的去留与报告方式，不动任何判据与门槛。

1. **第一节读法 1 要抄的六行**（`git show 04ceaa904^:research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree-r6-r3.out` 第 11–16 行，整行抄）：

```
E7RESULT name=frozen_tree key=file_difference against=first_segment_tree file=crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs tree_sha256=67e7a5498425ec3f27379ba8f0e5cd666faaeaac0817006d6938956d6419a156 other_sha256=b1457bc6175fe3c2b3da50b5e3b5901c1c9a8d95ce3080fef48d2c087bbe6d84
E7RESULT name=frozen_tree key=file_difference against=q5_tree file=crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs tree_sha256=67e7a5498425ec3f27379ba8f0e5cd666faaeaac0817006d6938956d6419a156 other_sha256=dbef78ed3eede487d5e5bea46dc5eb0a58735faa6930ea6548889bc62d908ea2
E7RESULT name=frozen_tree key=device_sha256 place=tree sha256=67e7a5498425ec3f27379ba8f0e5cd666faaeaac0817006d6938956d6419a156 expected=67e7a5498425ec3f27379ba8f0e5cd666faaeaac0817006d6938956d6419a156 holds=true
E7RESULT name=frozen_tree key=device_sha256 place=q5_tree sha256=dbef78ed3eede487d5e5bea46dc5eb0a58735faa6930ea6548889bc62d908ea2 expected=dbef78ed3eede487d5e5bea46dc5eb0a58735faa6930ea6548889bc62d908ea2 holds=true
E7RESULT name=frozen_tree key=device_sha256 place=first_segment_tree sha256=b1457bc6175fe3c2b3da50b5e3b5901c1c9a8d95ce3080fef48d2c087bbe6d84 expected=b1457bc6175fe3c2b3da50b5e3b5901c1c9a8d95ce3080fef48d2c087bbe6d84 holds=true
E7RESULT name=frozen_tree key=device_sha256 place=repo sha256=ab35105f861d509457296741d2c60515a3a94b9fefe77944f03d1b358afd45df expected=ab35105f861d509457296741d2c60515a3a94b9fefe77944f03d1b358afd45df holds=true
```

   两行 `file_difference` 都只点名装置一份，读法 1「修订 6 树等于新树只换装置」在这份产物上成立。Z42 要记的：两份冻结树产物的 sha256 与行数就是 A42 那两组，`build_approximate_tree.py` 与 `approximate_tree_lines.py q1` 每次读之前从 `df5866f67…`（`04ceaa904^`）现取、核这两组，不等退 3、什么都不造；第 1–13 步那几份留给第二段第一次取时记。近似树甲乙两份逐文件清单的 sha256（造树产物 `name=summary` 行的 `manifest_sha256=`）与修订 8 那一趟的导出根、产物前缀：产物还没跑、修订 8 那一趟还没跑，这一条不写。

2. **读法 3 的四类在逐份归类时怎么落**（只影响 Q1-未钉的描述份数；十五份钉值的归类照 A44 不变：装置 1、产品代码与池级 checker 3、别的验证包 11）。依据：读法 3 列的四类没有覆盖这三处，94 行与 T1 路径集里都有它们。
   - `crates/singlefs-checker-tier/src/bin/` 下全部归「装置」：第四类明写「`crates/singlefs-checker-tier/` 的非 `bin/` 部分」，而 94 行里 `bin/` 下还有一份 `e171_three_oracle_collusion.rs`（`M_`），别的类都不收它。
   - `crates/singlefs-replay-prefix/` 归「产品代码与池级 checker」（它的 `Cargo.toml` 照「各 `Cargo.toml`」归别的验证包）：`singlefs-core` 的恢复路径调它（它 `Cargo.toml` 的 description）。收严：第三类只宽不窄，Q1 往候选二翻只会更容易。
   - `crates/mutations.tsv`（T1 另带、94 行里 `MM`）归「别的验证包」。

3. **研究一侧十一份的找法落到路径上**：钉值表取 E171 研究一侧 `revision_8_research_side_of_the_revision_6_run.json` 在提交 `f4605214d` 的那一版（sha256 `2fab8fad7077c41113698666673533cc928cd25ea167122a1bf4e482f718f099`，与今天主工作区那份相同；那个目录有别的执行员在改，读提交里的）。表里 `s2_compare.py` 一项记的是仓外草稿目录里的那一份，按文件名对到 E171 研究一侧目录的同名路径，照那条路径在 git 里找各版，产物只写仓里的路径。两张坏实现表 T3 找不到时记 `missing`，不取今天那两份（读法 4 倒数第二条「今天主工作区里改名改内容的那两份不用」的字面）。

4. **T1 不收 94 行里的四份 `.wgsl`**（照读法 4 T1 的字面，不改路径集）：它们只被 `crash_judge_gpu.rs` 的 `include_str!` 读，那个模块在 `31e47115…` 的 `crates/singlefs-checker-tier/src/lib.rs` 里挂在 `#[cfg(feature = "gpu")]` 下；编两个 bin 不带 gpu 特性，编不到它们（读码推的，没编过）。编不过而诊断点到它们的，照 S41 停、交主 agent，不补文件。

5. **第六节「可跑性与判别力」第 3 条与第十四节「预计挂钟：」第二段那一句改写**。依据：派发提示「关的是哪几条」第 2 条「核那一句的来源（E171 第 1 次重跑修订 6 第 6.4 节、修订 8 第 8.9 节）改成量与口径对得上的写法，跑前改登记照「跑产物之前修订臂或判据合法、留记录」写一条修订」；门禁 doc-experiments 的 preregistration-parallelism 格把「17 800 态」读成 800 态，算出约 10.9 分钟，与写的约 4.1 小时差一个数量级以上。
   - 来源核对：`research/prompts/e171-r1-prereg.md` 第 1245 行（修订 6 第 6.4 节步表第 7 步）原文「旧树第 7 步 `process_wall seconds=14581.558`（调度记录第 1077 行转引）；主 agent 给的流三每态 0.82 秒（带探针，推的）与它同源，这里直接用进程挂钟」；同文件第 1648 行（修订 8 第 8.9 节）只引「之后第 7 步约 4.1 小时（修订 6 第 6.4 节）」。量的是第 7 步的进程挂钟；每态 0.82 秒是推的、与它同源；「17 800 态」是两数相除折出来的（14581.558 ÷ 0.82 ≈ 17782），不是数出来的态数。
   - 改法：进程挂钟写成量的、态数与每态耗时写成推的，数不用空格分千位。挂钟估值不变（第 7、9 步各约 4.1 小时，第二段约 14–16 小时），不动判据，不是放宽。
   - 原文照录。第六节那一条：「- 各步的态数与耗时照 E171 第 1 次重跑修订 6 第 6.4 节：流三约 17 800 态（推的：14 581.558 秒 ÷ 每态 0.82 秒，带探针），第 7、9 步各约 4.1 小时；不带探针的带流三单元 8 128–9 698 秒。」第十四节「预计挂钟：」里那一截：「第二段约 14–16 小时（推的：第 7 步流三约 17 800 态 × 每态 0.82 秒 ≈ 14 600 秒 ≈ 4.1 小时，第 9 步同；」

6. **T5 的「后代」与「加它的提交」落到命令上**（读法 4 没写到这一层，照字面最窄的读法写死）：后代是 `git log --all --reflog --ancestry-path ^31e47115…` 列的提交（分支、标签、stash 与 reflog 点到的，连同它们之间的提交），按（提交时刻, 提交号）取最早；改过的路径取「这个提交里有这条路径、blob 与 `31e47115…` 那一版不同」的最早一个；新文件取「这个提交里有这条路径、它的各父提交里都没有」的最早一个。T3 的「每一版」是 `git log --all --reflog --full-history -m --root --raw -- <路径>` 列出的每个 blob；悬空 blob 是 `git fsck --unreachable --no-reflogs` 列的 `unreachable blob`（fsck 退非 0 时照收，退出码进产物）。

7. **编法**：`cargo build --release --locked -p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes --bin e170_o3_spec_executor_export`，在近似树的 `tree/` 里起，编译目录取统一任务服务给的 `CARGO_TARGET_DIR`，编完拷进树里 `target/release/`（E171 第 0 步判法找二进制的位置）。不带 gpu 特性（第三节）；不带 `--offline`（E171 修订 8 第 8.11 节第 1 条记第二台 cargo 缓存缺 `vstd`，任务两台都可能派到）；带 `--locked`：`Cargo.lock` 对不上清单时编不过而不是改树（读法 4「不为编过而改」），编之前编之后核树的快照摘要相同。收严。

8. **第一段做到哪**：第五节 5.5 第一段写「写造树与第 0 步判法、单测、变异」；这一段的第 0 步判法只写 Q1 与 Q2-编那几行（`approximate_tree_lines.py` 的 `q1` 与 `compile-lines`）。A12 十二项、`stream-constants` 回比（S41）、S45 与两个二进制 sha256 的起跑核对（Z40 的二进制那一半）留给第二段与跑法一起写。依据：派发提示「关的是哪几条」第 3 条只要「照登记 T1–T5 造近似树、比 Q1 …，在树里只编两个 bin」。只记分段，不动判据；变异表这一段只有 M40–M47（第九节 M48–M52 管第二、三段的对照与归因，那时追加）。

### 修订 2（2026-11-02，用户定案，写在造树甲乙与单测、变异都跑过之后）

时点：第一段的单测 38 条、变异 8 条、造树甲乙两份产物都已跑完（产物在 `research/results/e200-revision-6-segment-one-on-an-approximate-tree-s1-*.out`），这一条写在产物之后，不是产物之前。依据：派发提示「关的是哪几条」第 1 条（「照这条写登记修订 2（第十二节，连同两份 `manifest_sha256`……）」）与那一条引的用户 2026-11-02 弹窗回答。这一条补的是第十一节 S40 写的「交主 agent」之后主 agent 经用户定下的记法；第一节读法、第六、七、九节、第十节、第十一节 S40 的触发观测与原文一个字不动，S40 打中的两份造树产物仍不当结果。

1. **用户的定案**（问句「E200 接下来怎么走？」，选项原文）：「Q1 记「差别伸进产品代码与池级 checker」（那 3 份已经定了这一边，缺的那份测试另注，推的：它翻不动候选）；Q2 记「近似树造不成，出不来」；E200 收尾，写进登记修订。不再跑 14–16 小时的第二段。」
2. **Q1 记候选二「差别伸进产品代码或池级 checker」**。依据（量的）：造树甲乙两份产物的 `name=file place=tree` 行，对存档钉值 `61da68ec…`（`crates/singlefs-core/src/allocator.rs`）、`4b716162…`（`crates/singlefs-checker/src/image.rs`）、`438b7187…`（`crates/singlefs-checker/src/walk.rs`）：甲取 `f8a9ddaab` 的版本，sha256 依次 `97af5769…`、`d325121c…`、`3a67063a…`；乙取 `31e47115…` 的版本，依次 `e25aa237…`、`67be92f9…`、`36ff699a…`；三份在两棵里都不等于钉值，读法 3 有一份落在产品代码与池级 checker 就判候选二。另注：缺的那一份是 `crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs`（钉值 `303dade9…`，别的验证包），仓、git 历史与悬空 blob 里都取不到，两棵树里没有它，Q1 够判条件「13 份逐份比完、每份写出近似树取的是哪一版与它的 sha256」在这一份上写不出；记法由用户定，不是这个够判条件成立了。推的：这一份归别的验证包，记不等还是不计都翻不动候选二；能翻它的只有在 T3 搜过的范围之外找到那三份钉值的内容。登记的 Q1 判法产物（`approximate_tree_lines.py q1`）不出。
3. **Q2 记「近似树造不成，出不来」**（第五节 5.1 Q2 第三个候选「出不来」里写的三种原因中的「近似树造不成或编不过」一种，取其中「造不成」）：S40 停在造树这一步，路径集里有一份新文件记 `missing`，近似树不齐，用户定这算造不成。没做的：树里编两个 bin 没试，所以「编不过」没测；第 0–13 步没跑，对照·修订 6、对照·修订 8 两份对照产物没出，「出得来」的两个候选都没有依据。
4. **第二、三段不跑**。依据：第 1 条用户的话，与第五节 5.5「一行开着的都不剩就停，后面的段不跑」和第十一节够判停机「没跑的量逐条标够判后未跑」。逐条对应：Q1 判法产物 → Q1；树里编两个 bin → Q2；第二段（写跑法与对照·修订 6、近似树上第 0–13 步） → Q2；第三段（对照·修订 8 与归因） → Q2。第九节 M48–M52 管第二、三段的对照与归因，那几份没写，变异行不追加；Z42 要记的第 1–13 步各份产物 sha256 与修订 8 那一趟的导出根、产物前缀不取。
5. **第十二节开头要抄的近似树甲乙逐文件清单 sha256**（两份造树产物第 308 行的 `manifest_sha256=`，整串照产物抄；`grep -o 'manifest_sha256=[0-9a-f]*'` 对两份产物原样一致）：
   - 甲（`research/results/e200-revision-6-segment-one-on-an-approximate-tree-s1-tree-a.out`）：`manifest_sha256=55345887e04fcd3289c6cd4934de404fe493d73a00e5d8bdbb39c54f70e7520b`
   - 乙（`research/results/e200-revision-6-segment-one-on-an-approximate-tree-s1-tree-b.out`）：`manifest_sha256=2e883c96931057e8e9de6356aa142a85d9867abe11006e53223b04da83ebeec3`
6. 近似树的读数仍不进 E171 第 2 次重跑的任何判据、不替修订 6 那一趟的读数（第一节读法 9 不变）；E171 第 2 次重跑的 ②③ 照用户定的记「修订 6 树与导出造不回来」。

## 十三、读过的文件与跑过的命令

读过的（行号区间；grep 命中行也算）：
- `.claude/agent-common.md` 1–121；`.claude/singlefs-ai-sop/rules/test-discipline.md` 1–138、`.claude/rules/three-way-inference.md` 1–168、`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 1–182、`.claude/rules/mutation-sampling.md` 1–93、`.claude/rules/implementation-first.md` 1–19、`.claude/rules/test-script-discipline.md` 1–106（全文）；`command-safety.md`、`writing-discipline.md`、`verify-before-claiming.md` 的 `grep -n '^#'`。
- `research/prompts/e171-revision-6-approximate-tree-questions.md` 1–12。
- `research/prompts/e171-r1-prereg.md`：`grep -n '^#'` 全部命中行；第 1099–1303、1433–1712 行。
- `research/prompts/e171-r2-prereg.md`：第 1–307、410–436 行；`grep -n '①\|②\|③\|④\|对照脚本\|contrast\|近似\|7726cdd6\|H15'` 命中行（第 106、299、393、394、408、414–418 行，各截到 600 字）。
- `records/2026-10-31-删兼容旧版本代码.md`：`grep -n '^#'`；第 1 行起约 25 行（工具输出预览）与第 168–181 行。
- `/home/fy5090/.local/share/singlefs/work/compat-removal-e171-r2-run-f/result.md`：`grep -n '^#'`；第 1–23、162–180 行；那个目录的 `ls`。
- 两份冻结树产物（`git show 04ceaa904^:research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree.out` 与 `…-frozen-tree-r6-r3.out`，取到草稿目录）：键种计数；前者 `key=git_status` 94 行、`git_head`、`snapshot_digest`、`git_status_lines` 与 13 行 `file_sha256`（第 99–111 行，各截到 200 字）；后者 `git_head`、三个快照摘要、`device_sha256` 四行、`p6`、`file_sha256` 两行、`binary_sha256` 两行（各截到 300 字）。
- `scripts/kb/claim-experiment.sh` 第 1–60 行（去空行）；`scripts/kb/quote-kb.py --help` 前 30 行；`scripts/agents/snapshot-tree.py` 第 1–40 行（去空行）。
- `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/`：`ls`；`row_driver.py` 第 225–275 行；`contrast_revision_6_against_revision_8.py` 第 1–30 行与第 9、15、18、54、55、58、127–136 行；`segment_one_units_revision_11.py` 第 60–204 行里 `grep` 命中行；`frozen_tree_lines_revision_8.py` 第 2–45 行里 `grep` 命中行；`revision_8_research_side_of_the_revision_6_run.json` 第 1–40 行；`wc -l` 五份。
- `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`：`sha256sum`、`wc -l`、`grep -c`；同路径在 `f8a9ddaab` 的那一版：`sha256sum`、`wc -l`、`grep -c`；`git show 31e471154:crates/singlefs-checker-tier/Cargo.toml` 的 `grep -n gpu`（第 19、20、27 行）。
- `.claude/heavy-thread-default` 全文；`.claude/gate.d/doc-experiments.sh` 的 `gate-cell` 名与第 27、28、46–48、147–158 行；`scripts/gate/wallclock-assert-lint.py` 第 1–60 行（去空行）；`research/prompts/e199-preregistration.md` 第十四节。

跑过的命令（原样；`<草稿目录>` = `/home/fy5090/.local/share/singlefs/work/compat-removal-e171-approximate-tree-design`）：

```
$ bash scripts/hosts/work-directory-ownership.sh claim <草稿目录> --purpose 'E171 修订 6 那一趟在近似树上重跑的跑前登记'
$ bash scripts/kb/claim-experiment.sh --next          # E200
$ bash scripts/kb/claim-experiment.sh E200 修订6近似树重跑
$ git ls-tree --name-only 04ceaa904^ research/results/ | grep 'e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-0[5-7]'
$ for s in frozen-tree frozen-tree-r6-r3; do git show 04ceaa904^:${P}$s.out > <草稿目录>/archived-$s.out; done; wc -l …; sha256sum …   # 125 行 b720e5d2…；62 行 786e4801…
$ grep 'key=git_status line=' <草稿目录>/archived-frozen-tree.out | sed 's/.*line=//' > <草稿目录>/git-status-94.txt; cut -c1-2 … | sort | uniq -c; grep -c -E '\.rs$|Cargo\.(toml|lock)$'   # 89
$ git ls-tree -r --name-only 31e471154 -- Cargo.toml Cargo.lock crates | grep -v '/target/' | awk -F/ '{n=$NF} n ~ /\.rs$/ || n ~ /^Cargo\./' | sort > <草稿目录>/head-digest-paths.txt   # 270
$ grep -E '^(A_|AM|\?\?)' <草稿目录>/git-status-94.txt | cut -c4- | grep -E '\.rs$|Cargo\.' | sort > <草稿目录>/added-paths.txt   # 14；comm -12 交集 0；并集 284
$ git merge-base --is-ancestor 31e471154 HEAD   # 是祖先
$ git log --all --format=%h -- <rows_orchestrator.py 与 run_segment_one_on_the_revision_6_tree_multi_host.sh> | while read c; do git show $c:<路径> | sha256sum; done | sort -u   # 各一版：e6f3a370…、732473a1…
$ git show f8a9ddaab:crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs | sha256sum / wc -l / grep -c 'crash_amplification::\|crash_state_enumeration::'   # ab35105f…；12645；0
$ grep -rn 'approximate_tree\|approximate tree' crates/ | wc -l   # 0
$ git log --all --diff-filter=A … -- 'research/prompts/*-snapshot/judged-vs-*.diff' 'research/prompts/*-snapshot/*.diff'   # 2026-10-03 到 10-20 之间只有 test-script-align 两份，没有 crates 的
$ python3 scripts/kb/quote-kb.py <草稿目录>/quotes.md 'research/prompts/e171-r1-prereg.md@#### 8.5 两趟之间的对照' 'research/prompts/e171-r2-prereg.md:198-198' 'research/prompts/e171-r2-prereg.md:299-299' 'research/prompts/e171-r2-prereg.md:414-418'   # 4 段，回读逐字节一致，43 行
$ TZ=Asia/Tokyo date +%F   # 2026-11-02
$ cat <草稿目录>/part-*.md 依次 >> research/prompts/e200-preregistration.md（每段不超过 150 行），第二节在 part-3a 之后追加 quotes.md；追加后核见报告
```

## 十四、并行与挂钟

独立单位：造树按路径独立、算 sha256 按 blob 独立；近似那一趟照 E171 第 1 次重跑修订 8 第 8.4 节的单元（第 3、4 步每项、第 10 步每次、第 11–13 步每行各一个）；对照按步、按比法独立；近似树乙与近似树甲互相独立。
线程：取登记值 `.claude/heavy-thread-default` 的 31；每条经统一任务服务交的命令由服务给线程与内存，登记里不写。
串行：第 0、1、2、2b 步依次跑（后面全部单元的前提，第 2、2b 步吃前一步的导出）；第 8 步在第 7 步之后（读它的导出）；第 11–13 步在第 0、1、2、2b、7 步都绿之后（逐行限时要第 7 步的 T）；第 7 步不切（T 定义成它的进程挂钟）；第三段在修订 8 那一趟第一段跑完之后。
预计挂钟：第一段约 1 小时（推的：造树约 1.9 万个 blob 算 sha256 估 2–5 分钟；树里 release 编两个 bin 约 20–40 分钟，依据 E171 修订 8 第 8.9 节；单测与 13 条变异约 15 分钟）；第二段约 14–16 小时（推的：第 7 步带流三、带探针，照 E171 修订 6 第 6.4 节旧树量的进程挂钟 `process_wall seconds=14581.558` 约 4.1 小时，第 9 步同；流三每态 0.82 秒是与这个进程挂钟同源的推算、态数也是两数相除折出来的：17782 态 × 0.82 秒 ≈ 14581 秒（第十二节修订 1 第 5 条）；第 3 步带流三两项各约 2.3–2.7 小时与它们同时跑；第 11–13 步 16 行，带探针两行各约 4.1 小时、不带探针十四行各约 2.3–2.7 小时，单道合计约 43 小时，照 E171 修订 8 第 8.9 节当时的分配推约 9.5 小时；近似树乙第 0、1 步约 25 分钟）；第三段几分钟到半小时（推的，导出大小没量，第一节未定项）。第二段超过 2 小时：分片单元是上面那几种单元与行；续跑入口是同一条命令重起，只补没做完的单元（E171 修订 8 第 8.4 节「续跑」的三样核：日志里有绿行、产物在且行数同、单元绿时写下的清单 sha256 不变）。
预计在用核数：第一段编译时 31 个核（一条任务用登记的 31 线程），造树 1 个核；第二段每条任务 31 线程，同时在跑几条由服务定，推的是服务给的全部重活线程都占满（依据：第 3–10 步十几个单元可以同时起，第 11–13 步 16 行）；第三段 1 个核。

重型的几步：近似那一趟第 0–13 步与近似树乙第 0、1 步（跑 checker 档包里的实验装置，标重型）；第一段树里 release 编两个 bin（只编不跑，照 `.claude/hooks/heavy-test-guard.sh` 文件头 build 那一类判）；造树、对照、归因与单测、变异是研究一侧 Python，不是重型。
