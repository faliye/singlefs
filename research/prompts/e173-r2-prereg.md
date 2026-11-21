# E173 重跑登记（第 2 次）：入库装置第一段在今天的 `crates/` 上重出，只重跑复跑表 `replay.sh` 第 2921 行

写于 2026-11-13，装置不改、新产物一次没出之前。不占号。上几次登记：原登记 `research/prompts/e173-preregistration.md`（已归档，`git show 04ceaa904^:research/prompts/e173-preregistration.md`，`sha256sum` = `1c9b1de446b1803f84995aef06425a4b609312e9ab6f55f56c9f2f45118fdaf8`）；第 1 次重跑 `research/prompts/e173-r1-prereg.md`（已归档，`git show 59ce5a865:research/prompts/e173-r1-prereg.md`，`sha256sum` = `c32cfc4abef788ce0147c4edbcd2f75794f3c64206869ebff96aab5c34846fb4`，含装置写之后补的第十二节修 1–修 20）。下面「同 r1 第 X 节」一律指取出来的这一份。

这一次是差分重跑登记（`experiment-designer` 定义第 4 步）：臂、判据、岔路都不变，变的是输入（`crates/` 里装置读的产品代码）。所以只写三样：变了哪些输入（第四节）、哪几行判据要重判（第一、六节）、比对对象与产物名（第十一节）；其余各节写「同 r1」并引节号，不重抄第十三节之前 r1 的长表。节名照固定形态全留。

<!-- doc-lint:not-numbers Q1 Q2 X1 X2 X3 X4 X5 X6 S2 S3 S4 S6 V1 V3 V6 PC7 A1 A6 A7 A9 B10 B11 G-min G-1 G-4 MR1 MR2 MR3 MR4 MR5 MR6 MR7 MR8 R1 R2 RV1 RV2 RV3 RV4 RV5 RF1 RF2 RF3 RF4 -->

要主 agent 认的项：①问法读法 A（第一节「读法写死」：问题二只答第一段读数能答的那一半，第二段的数不在这一次重判）；②复跑表第 2921 行换名之后「比对对象表」与机读表 `.claude/gate.d/replay-comparison-units.tsv` 的登记（第十一节末）；③准入没有在设计员这一侧判出（设计员不起任务，第四节末与报告「要主 agent 跑的命令」）。

## 一、问题

英文名：checkpoint_cost_against_empty_publish_cost（照抄 r1 第一节；装置是入库装置 `crates/singlefs-implementation-experiments/src/bin/e173_checkpoint_cost_against_empty_publish_cost/main.rs`，r1 写成 `crates/singlefs-checker-tier/src/bin/…` 的那一处，今天在 `crates/singlefs-verification/` 下，路径迁移登记在 `.claude/gate.d/path-moves.tsv` 第 26 行；装置不改，不另起新装置）。

主 agent 给的问题（派发提示逐字）：

> E173（上限卡在读坏槽的三种处置） 的留存产物 `research/results/e173-checkpoint-cost-against-empty-publish-cost-2026-11-02.out` 在今天的代码上对不上：另一个会话的实验复跑任务 13487 报它 4 行不同（留存 `NoSpaceAfterRaisingTheFloor`，现跑 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`）。主 agent 此刻量到留存产物里 `NoSpaceAfterRaisingTheFloor` 出现 2 行、`NoSpaceEvenWithEverything` 0 行（`grep -c`）。要一份第 2 次重跑的登记：只重跑复跑登记表 `scripts/experiment-apparatus/replay.sh` 第 2921 行那一条（`@driver_e173_checkpoint_cost`），别的三行（第 2918–2920 行）不重跑、写明为什么。验收：登记写死臂与判据不变（照实验页原登记，不新加臂）、这一次重跑要回答问题单的两行、作废条款、产物落到哪个新文件名（旧产物不覆盖）、复跑登记表那一行怎么换。

问题单（`research/prompts/e173-r2-questions.md`，逐行抄）：

| 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 一、留存产物与今天的代码差在哪 | ① 只差被拒的结局名（原来报推满仍放不下的格，今天有的改报先算判出放不下）；② 除了结局名，还有计数、写数、屏障数、空发布次数这类量变了 | 逐行比今天现跑的输出与留存产物：有没有一行的数值字段不同 | 那一条驱动在今天的代码上跑完，与留存产物逐行比，列出不同的行与不同的字段 | 开着 |
| 二、E173 的结论要不要改 | ① 问题单第 1 行那一条判语（按 ckpt_cost 现算的代码读法与条款读法不同、只代码读法打中的格数）照旧；② 判语里的格数或哪一臂打中变了 | 重跑之后判语里引的每个数与实验页现有的是否相同 | 第一行够判之后，对着实验页「结论」逐个数核 | 开着 |

**读法写死**（读法 A；派发提示未另给读法，设计员取的，待主 agent 认）：

- 「逐行比」：新产物与留存产物都先去掉准入头、负载字段与计时字段（`replay.sh` 的 `strip_admission_header`、`strip_load_fields`、`strip_timing` 三个函数在 `scripts/experiment-apparatus/replay.sh` 第 5212–5225 行用的就是这一套，`diff` 的比法照它），再逐行逐字比；差一行记「一行不同」，与任务 13487 报的「4 行不同」的口径（`diff` 的 `<`、`>` 行合计，一行变了数两次）一致：留存产物里含旧结局名的 2 行变了，数出来就是 4。
- 「结局名」：一个字段值里以字母或下划线起头、只含字母、数字、下划线、冒号的最长一段，且含子串 `NoSpace`（`UserChangeRefused` 的成员名，装置的 `variant_name_of`，`e173_checkpoint_cost_against_empty_publish_cost.rs` 第 252–262 行，只取成员名几层、不带数）。「仅差结局名的行」：新旧两行把各自所有这样的段整段换成同一个占位符之后逐字相同。其余的行不同都算「别的字段不同」，按字段名（`key=value` 里的 `key`）列出。
- 问题一的两个候选：候选 ① 成立 ⇔ 全部不同行都是「仅差结局名的行」，且新产物行数等于留存产物行数、末行 `E7RESULT name=done emitted=N` 的 N 相同；候选 ② 成立 ⇔ 有任一行有「别的字段不同」，或行数不等，或行序里某行缺了/多了。两个候选互斥；都不成立（比对跑不出来）按「没跑」报。
- 问题二「引的每个数」：只取第一段（本装置）产出的、实验页第 1 次重跑一节引到的数——第四节列的那张清单（按几何、按字段）。第二段（`replay.sh` 第 2922 行，`scripts/experiment-apparatus/` 下独立手写的模型，不读 `crates/`）与它给的数（Z1 的 152 格、Z3 的 1 格、Z2 的 33 格这一类）这一次**不重测**：读法 A 下问题二的 ① 只对第一段那份清单判；第二段的输入（9 个几何的 (S, c实, κ) 常量表，抄自第一段产物）有没有变，由第六节 Y 量判。若主 agent 要第二段的格数也重测，那是读法 B，要另写一份重跑登记（第二段装置没改、输入也没变时，准入会判「没变」退 77，见第十一节）；两种读法都不改变本登记任何判据，只差报告里问题二能不能写「判语里的格数照旧」——读法 A 下只能写「第二段输入没变，所以第二段的数照旧」，那一句是推的、不是测的。
- 其余（几何、历史、读法 κ 与 c实 的定义、G-min 的找法）逐字同 r1 第一节「读法写死」与第五节「第一段要做的」。

未定项：无（三样可跑性与判别力都核出来了，见第四节与第十四节）。

## 二、被测条款与它引的定义

被测条款是 `.claude/kb/decisions/16-发布语义.md` 已定项 10「射程」段里「抬 F 之前先算」那几句（今天写回过的现文，下面整段抄）。这一次重跑装置读的是 `crates/` 的产品代码，它被这几句改动的是被拒的结局名（`NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`），不是条款对 E173 第一段的任何判据；E173 本身的被测条款与定义（D16（发布语义） 已定项 1「准入」「`df`」两行、D28（挂载期承诺量） 已定项 4）同 r1 第二节，不重抄。

**出处 `.claude/kb/decisions/16-发布语义.md:270-293`（整段抄，未转述）**

```markdown
#### 已定项 10：发布粒度取攒批，不是每个事务发一次

**实现边界**：攒批的会话是 `crates/singlefs-core/src/mounted_session.rs:4735` 的 `MountedSessionWithPendingChanges`，四个入口是 `crates/singlefs-core/src/mounted_session.rs:5249` 的 `accept`、`crates/singlefs-core/src/mounted_session.rs:5472` 的 `publish_when_due`、`crates/singlefs-core/src/mounted_session.rs:5490` 的 `publish_pending_now` 与 `crates/singlefs-core/src/mounted_session.rs:5508` 的 `publish_user_change`（先排空待发）。收进待发时同一 inode 的整份写折叠成最后一次、建 inode 按收下的次序追加（`crates/singlefs-core/src/mounted_session.rs:4797` 的 `take_in`），待发整个并成一次发布走 `crates/singlefs-core/src/transaction.rs:10051` 的 `publish_data_changes_published_together`；屏障进门先排空、自己单发（`crates/singlefs-core/src/mounted_session.rs:5279`）。待发里已有另一个 inode 的整份写时，`accept` 在任何读写之前拒成 `WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion`（`crates/singlefs-core/src/mounted_session.rs:5342`）；按偏移写与已并好的一批也在进门时拒（`crates/singlefs-core/src/mounted_session.rs:5282`、`crates/singlefs-core/src/mounted_session.rs:5286`）。不经会话的 `crates/singlefs-core/src/mounted_session.rs:2100` 的 `MountedSession::publish_user_change` 仍每次改动同步发布。

**定案**：攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次。会话的入口：`accept` 把一个改动收进待发；`publish_when_due(now)` 由调用方按周期调，到触发就发；`publish_pending_now` 立即排空；原 `publish_user_change` 语义不变，先排空待发再做它自己。数据写类可并：同一 inode 的写折叠、建 inode 求和；unlink、截断、建删快照、整理、回退、卸载是屏障，进门先排空待发、自己单发。`accept` 进门先做这一批合起来的准入与条数预判，过不了就在任何读写之前拒，不在返回成功之后拒；发布写失败冻结之后 `accept` 在任何读写之前拒。

**射程**：定的是频率，不定一次发布事件内部的顺序——那在 `litmus/commit-publish.litmus`，它钉的是写块内容 → `smp_wmb` → 写 generation，对频率一字未提。这一版不并行（用户 2026-10-31 定）：两个 inode 的写不并进一次发布（`accept` 拒成 `WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion`），并发读写与并行提交归里程碑六（[06-sixth-txn.md](../milestone/06-sixth-txn.md)）。`accept` 被空间拒时自己抬 F（用户 2026-11-09 定）：与 `publish_user_change` 同一个推法、同一个上限，推满仍放不下才拒；条数预判过不了与冻结照旧在任何读写之前拒。抬 F 之前先算（用户 2026-11-09 定）：推抬 F 放得出的全放出来也放不下就一串都不推，报 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，头一次被拒就判出时盘上逐字节不变（推过一串之后、或 unlink 意图发成之后走空才判出的，盘上有这次调用写的东西）；先算罩 `accept`、`publish_user_change` 与走空三处，不罩改动发成之后另发的搬迁批、挂载续做与挂载准入的推串，也不判提交内生块那一判（照推）；短缺种类混着来时可能推过一串才判出；式子那一判的上界是可用加 defer 加隔离；短在段外槽对那一判同样先算，上界是段外两槽都不罩在未释放分配记录下的槽对数（defer、隔离、抬 F 扣住三样全放开之后成的对，现有的在里面；分配器随每次改未释放记录按段增量维护，准入不扫单元区），上界恰好等于需求照推。这次调用还一串没推、被拒是空间不够（空间准入被拒与取落点被拒两支都算）、被拒的是合取 7 判的前台写入（写文件内容、建 inode、并进一次发布的数据写类；改 size 与建快照不先判，照推）时，写之前那一刻全空聚簇段已少于 N₀（合取 7 自己读的那个数）也一串不推，报同一个成员、带合取 7 那一份；先判槽对与式子，再判合取 7，两样同时成立只报槽对那一判。先算拒绝时报里不带卸载重挂这条出路（用户定另立一题，做成调用方显式发起的查询，欠在 C1003（被宽护卡住那条出路没量过））。误拒的读数（探针造的池上，对照臂是不先算、照推的那一臂）：槽对那一判在对照臂推过至少一串的 1082、722、705 格里推完写进的都是 0，合取 7 那一判在 303、386 格里写进的都是 0；这是几份取样里没撞上，不写成全称。先算之前量到的连续被拒推串的代价：每次 `series=2 empties=6`，一次白写 `writes=40 barriers=16 written_bytes=500736`；状态数掉下去不是推串磨掉的（`write=Nothing` 同样在第 4 次崩溃掉到 1）。两样已知边角：

- **净代价分布**：格式侧几乎为零（journal 本来就要有），验证侧一大笔——中间态语义改写（已定项 12）、第二份 replay 实现（已定项 4 与 13），再加 D13（验证路线） 的「Merkle 根链差分」粒度变粗：一代根里塞了 K 个事务的改动，定位精度从「哪个事务」降到「哪个 checkpoint」，而 D13（验证路线） 已写明不指定范围那一趟的真代价是定位成本。
- **发布语义决定「走几次」**：每事务发根 → K 批走 K 次；checkpoint → 同一窗口内的 K 批只走约一次，各批只留一条 journal 记录（小、顺序写）⇒ 分批大删除那一格的元数据写放大从 `K × M × H` 降为 `checkpoint 数 × M × H`。

**依据**：

- 无实验：定案靠两条互相独立的推理，当时没有可量的量。① 它与 D5（快照 / 空间记账机制） 已定的 ZFS birth txg 模型天然相容——ZFS 的 txg 本身就是 checkpoint（`zfs_txg_timeout` 默认 5 秒，或攒够一定脏数据量），一个 txg 批量大量操作、txg 结束才更新一次 uberblock；采纳 checkpoint 之后 birth txg 的号粒度问题自动消失，一个 txg 一个号，deadlist 的比较粒度就是 checkpoint 粒度、不必重算（未在本项目验证）。② **根是丢失写的最终见证者**：检测「盘上这块是不是它应该是的那个版本」需要一个外部见证者，块自身办不到（合法的旧版本与合法的新版本在块内层面同构，D18（块里携带什么信息） 有完整论证），而见证者必须在不同的失败域上——父节点若在同一次事件里也没写成就要祖父，递归到根 ⇒ checkpoint 发布的那个根是这条递归的终点，它的持久化必须比任何单个块更可靠（ZFS 的形态是 4 个 label × 每个 128 KiB 的 uberblock 环、且偶数与奇数 label 分两阶段写；本工程根自身的冗余形态由 D22（单元原子性怎么合成） 已定项 2 定）。频率的可量代价（两个阈值的可行域）在已定项 5。
- 判决 `research/prompts/m4-publish-trigger-r1-main-verification.md` 第二节「一次发布里攒什么」「返回即落盘」「串行下不立的」三格：屏障分类、进门预判准入与条数、冻结后拒、三个入口（两腿合，被攻过零轮）；用户定案 2026-10-26：射程「串行下时间与脏量两种触发」（原话在变更史）。
- 判决 `research/prompts/m4-close-code-r1-main-verification.md` 第四节 T4.5 那一行（两个号的写攒进一次发布，今天拒并，比条款窄，交用户）；用户定案 2026-10-31「这一版按照不并行」（无实验：用户定案，原话在变更史）。
- 判决 `research/prompts/m4-gpugc-mount-runtime-code-r3-main-verification.md` 第二节「连续 `accept` 被拒推不停」「状态数是 `accept` 磨掉的」两行与第三节第 1 条：`crash_loop_summary … write=NeverFits empties_in_total=186 refused_with_empties=31`；`write=Nothing` 与 `write=NeverFits` 逐次挂载的状态数同在第 4 次崩溃掉到 1。
- 用户定案 2026-11-09（`accept` 被空间拒时自己抬 F；抬 F 之前先算），原话在变更史。
- 用户定案，原话在变更史。
- 判决 `research/prompts/m4-gpugc-precheck-b-a-code-r1-main-verification.md` 第二节与第三节第 3 条、`research/prompts/m4-gpugc-precheck-b-a-code-r2-main-verification.md` 第二节与第三节第 5 条、`research/prompts/m4-gpugc-precheck-b-a-code-r3-main-verification.md` 第二节两张表与第三节第 3、4 条：改法乙（槽对那一判的上界）与改法甲（合取 7 那一形）的探针读数与有效分母，两样误拒都在取样里没撞上；先算只罩三处，先算拒绝时不报卸载重挂。探针是三方腿的模型、不是登记的实验，没有实验号（无实验：模型目录在各轮判决里点名）。
- 用户定案（开改法乙、甲；先算拒绝时不报卸载重挂，另立一题做成调用方显式发起的查询），原话在变更史。

**欠**：C1003（被宽护卡住那条出路没量过）；C1025（槽对上界多算活意图 R 里的槽对）；C1026（每次发布克隆分配器多一张位图代价没量）。

```

## 三、实现今天的样子

- 装置：`crates/singlefs-implementation-experiments/src/bin/e173_checkpoint_cost_against_empty_publish_cost/main.rs`（2832 行）。`main` 第 2427 行；准入判法 `preflight` 第 2365 行（头一句，`admission: always`）；本地常量回比 `check_local_constants_against_the_crates` 第 2088 行（停机条款 S4）；锚点 `check_the_anchors` 第 2182 行（V3）；每个几何的阳性对照 `positive_control_row_and_holds` 第 1744 行（PC7 的产物行）；G-min 的找法 `smallest_completed_trial` 第 2304 行与 `main` 第 2474–2491 行；一次试跑一行 `trial_row` 第 2412 行（字段 `end=`、`refusals=`、`first_refusal=`，被拒的结局名从这几处进产物，`variant_name_of` 第 252 行只取成员名几层、不带数）；X5 的 `excess_slots` 第 1897 行；`completes_the_history` 第 1184 行。
- 被测代码里这一次变了的地方：`crates/singlefs-core/src/mounted_session.rs` 的 `UserChangeRefused::NoSpaceAfterRaisingTheFloor`（第 657 行，推满仍不够）与 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`（第 661 行，抬 F 之前先算判出放不下，枚举成员的文档在第 658–660 行，结构体第 693 行）。装置不改。
- 复跑表：`scripts/experiment-apparatus/replay.sh` 第 2921 行 `E173|@driver_e173_checkpoint_cost||e173-checkpoint-cost-against-empty-publish-cost-2026-11-02.out|exact`；驱动 `driver_e173_checkpoint_cost` 第 4284–4286 行：`cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e173_checkpoint_cost_against_empty_publish_cost -- segment-one`。该驱动点到 `CHECKER_TIER_FEATURES`，按 `replay.sh` 文件头第 4–6 行的「按需领卡」由脚本自己领卡，登记不写卡。
- 这个装置不走崩溃判段（不调 `crash_amplification` 的 `run_from_environment` 一族、`GivenStatesEngine` 一类入口），登记第 3 步那条「`task-server-gpu-arguments` 头」与「按 `--features gpu` 起」对它不适用，起法就是复跑表那一行；装置自己也不打 `NOT_RUN` 行，第十一节仍留一条 `grep -c NOT_RUN` 当防线。
- 零命中记录：装置里没有 `NoSpaceAfterRaisingTheFloor`、`NoSpaceEvenWithEverything` 字面串（`grep -n 'NoSpace\|ENOSPC\|Rejected' …e173_checkpoint_cost_against_empty_publish_cost.rs` 零命中），结局名只经 `Debug` 文本进产物；所以装置一行没改，产物里名字的变化只来自 `crates/singlefs-core` 的成员重新分了类。

## 四、跑之前已经存在的数

下面每一条都是读定义、判问法与核可跑性时已经知道或算出的数，照实列，没删。

1. 主 agent 的 `grep -c`（派发提示给的，没复核）：留存产物里 `NoSpaceAfterRaisingTheFloor` 2 行、`NoSpaceEvenWithEverything` 0 行。任务 13487 报「4 行不同」：按第一节口径，一行变了数两次，2 行变 = 4。对判据的影响：候选 ① 的预期形态是「2 行变、且只变结局名」；这 2 行是不是还带别的字段变了，没人看过（本设计员没读过产物内容，只取了 `sha256sum` 与 `wc -l`，见第 3 条）。判据没因这两个数松或紧：候选 ① 要求所有不同行都是仅差结局名的行，对「2 行」或「别的行数」一样判。
2. 实验页（`.claude/kb/experiments/173-上限卡在读坏槽的三种处置.md`）第 1 次重跑一节我为了认口径读到了，其中第一段读数的数已经看见，这一次的比对对象清单从那里抄：第一段 X3：`s4_g_min_64mib` κ = 18、c实 最大 = 6、n代 = 3、n条 = 22；`s4_g_1` κ = 130、c实 = 6、n代 = 21、n条 = 153；`s4_g_4` κ = 543、c实 = 10、n代 = 54、n条 = 381；`s8_g_min_64mib`、`s8_g_1`、`s8_g_4` 三行的 κ、c实、n代、n条与 S = 4 那三行相同；页上另写「三个 G-min 几何上 n代 是 3、3、4，其余六个 ≥ 21，n条 九个都 ≥ 22」「S = 16 的 G-1 上 n代 = 21，一次挂载累计推空发布 22 次比它多」（页上写 22 次出自第二段 Q1-3 的读数、21 出自第一段 3460 行，22 不在这一次重测）「X5：`s8_g_min_64mib`、`s16_g_min_128mib`、`s16_g_1`、`s16_g_4` 四个几何上 `excess_slots=2`」。S = 16 三行与 `s16_g_min_128mib` 的 κ、c实 页上没抄全，不列，比对时对着产物里整行比。对判据的影响：问题二的判据是「新数与这些数相同不相同」，这些数是比对对象，不是门槛，不进臂的定义，所以不是同义反复。
3. 留存产物只取了元数据，没读内容：`sha256sum` = `85bed2435162a532566e64a1fc73bcaf4ab1f8c6a26b1b7a03c76135c89b07d9`，`wc -l` = 3523，字节数 919531，写于 2026-11-02（`ls -l`），已入库（`git ls-files --error-unmatch` 通过，提交 `4964b76b8`）。与实验页写的 3523 行、sha256 前 16 位 85bed2435162a532 一致。
4. 输入变了的证据（现查）：装置源码 `sha256sum` = `a4cef9511eba14037d51d03e1bf5af26eb4c69f1f735d614f8e6afb366469887`；它读的 `crates/singlefs-core/src/mounted_session.rs` = `0bc41b0c655c6a5fd98d2a3e145793453a0b2e86af415f666168213f10e3f33d`（修改日期 2026-11-13，晚于留存产物的 2026-11-02）、`mount.rs` = `ae24940185ad01022d0054f631b2b710fe0e0f9cecba8750e352d873e67eba94`、`admission.rs` = `7cc59211bf6fcbee10c0de2abe8062235a55aa78d77fe228cf390df4f2eb0fa0`、`allocator.rs` = `eb822f0920329af40400071c5b2f306b3fe92e1899fdca8984f5edf65a426faf`；`git diff HEAD --stat -- crates` 末行「381 files changed」。r1 第十二节记的 2026-10-20 开工时旧哈希（`admission.rs` `e239a81f…`、`mount.rs` `655aaad1…`、`allocator.rs` `c376167f…`、`mounted_session.rs` `83cbe826…`）与这四个都不同。这只说明 `crates/` 在两个日期之间变了，不说两个哈希之间每一处改动都碰第一段。
5. 准入（`experiment-designer` 定义第 4 步）：判法命令是 `task-server/task submit -- python3 scripts/gate/admission.py experiment /home/fy5090/code/singlefs E173`，`task wait` 末行 `exit=77` 表示输入没变、不写重跑登记。**这一条设计员自己没跑**（不往任务服务交任务，`agent-common.md`「子 agent 不交任务」），所以没有结局可写；只有一条推的：派发闸（`.claude/hooks/runner-dispatch-guard.sh`）对「派 `experiment-designer` 写重跑登记而准入判输入没变（退 77）、退 2、前提没齐（退 3），提示里又没有一行『准入已判：<理由>』」的情形拒派；这次派发提示里没有「准入已判」那一行而派发放行了，由此推准入这次没退 77、2、3 中的任何一个。推的，不是测的；主 agent 的任务号与末行填在报告里。第 4 条的哈希与修改日期是独立于准入的旁证，说明输入确实变了。
6. 可跑性（第 3a 步 ①）：9 个几何（S = 4、8、16 × G-min、G-1、G-4），G-min 每个 S 另有五档试跑（被选中的那一档与 G-min 同一份读数）；r1 第十四节估约 3200 次发布，S = 16 时一个几何约 600 次发布；单态耗时没有量过，估的：r1 估第一段跑约 10–30 分钟（推的）；留存产物 919531 字节。线程取 `.claude/heavy-thread-default` 的 31；内存上限 r1 推的 20G。挂钟与核数见第十四节。量过的读数没有，要量的命令写在报告「要主 agent 跑的命令」一节，读数由承接的执行员填进第十四节。
7. 分辨率（第 3a 步 ②）：判据比的是产物的文本行，没有时钟、计数粒度或采样间隔的问题；要分辨的差是「结局名换了」与「某个数变了」，最小的差是 1 个字符，`diff` 的粒度是 1 个字节，够；判据不用相对变化、比值，不会在某一条臂的基线上退化。
8. 触发事件（第 3a 步 ③）：这一次的前提是被拒的结局出现在产物里。留存产物里含旧名的有 2 行（第 1 条），「某些历史里有一次被拒」在旧代码上已经发生；今天的代码上发不发生、报哪个名，正是问题一在问，是被问的量，不是前提，不另算式子。

## 五、臂、阳性对照、真实基线

**装置写在哪（写死）：入库装置**，就是 `crates/singlefs-implementation-experiments/src/bin/e173_checkpoint_cost_against_empty_publish_cost/main.rs`，不改、不另写独立模型。理由同 r1 第五节：问的是「`crates/` 这份代码在今天的真盘面上算出的 ckpt_cost 与一次空发布真写出的固定点，产物与上一次差在哪」，另写一份独立模型答不了。因为装置不改，也就没有「本地常量加回比」要新写：它读 `crates/` 的那几个常量（`SLOT_BYTES`、`DATA_UNIT_BYTES`、分配记录树的叶罩槽数与内部扇出、根环区数、每区槽数范围等）早已是本地常量加回比（r1 第五节），由装置开跑头一步 `check_local_constants_against_the_crates` 判，不符打 `name=stop clause=s4_local_constant_differs_from_the_crates` 退 3（第十一节 S4）。

**臂：没有**。这是同臂（r1 第一段本来就没有臂，r1 第五节「第一段要做的（入库装置，没有臂）」）、同输入类型下换了产品代码的重跑，不新加臂（派发提示：「照实验页原登记，不新加臂」）；r1 第二段两条臂（代码读法、条款读法）在模型上跑，不在这一次。不适用的格（臂的定义、支持各臂的人认的样子）写明：没有臂，这一格不适用。

**阳性对照**：没有臂可挂，阳性对照是产物自带的那一行：每个取样几何一行 `name=positive_control…`（`positive_control_row_and_holds`，PC7：路一把每个固定点角色记两次的钩子开着时 S3 判红、关着时两条路相同；覆盖写普通分配槽数 > 0、每次空发布 = 0）。装置在每个几何上都跑；没跑到或不成立打 `name=void clause=v1_positive_control_routes`（第十一节 V1）。比对脚本（第六节，这一次新写的一小段判定代码）自带四份造出来的输入当它的阳性对照，见第九节。

**真实基线**：2026-11-02 的留存产物（第十一节表第 1 行）；以及同一份输入在今天的代码上跑第二遍的产物（逐字节比，取舍同 `replay.sh` 的三个 `strip_` 函数，第十一节 RV5）。

## 六、报哪些量与各自的判据

每个量各占一行、各报各的判定，不合成一个真假。格 = 几何（9 个，同 r1 第一节）× 行；下面的量一律「逐几何」报，不聚合。门槛不是从臂的定义推出的（没有臂）：问题一的门槛来自问题单翻面观测的原话，问题二的门槛是「新数等于页上引的数」，数的来源是第四节第 2 条、不是装置的定义。

| 编号 | 量 | 怎么算 | 门槛与判定 | 对应问题单哪一行、取什么值翻面 |
|---|---|---|---|---|
| R1 | 不同行清单 | 新产物（今天）与留存产物各去准入头、负载字段、计时字段（`replay.sh` 的 `strip_admission_header`、`strip_load_fields`、`strip_timing`）后逐行比；每个不同行报：两边的行号、两边的整行、分类（仅差结局名 / 别的字段不同）、「别的字段不同」时列字段名与两个值 | 全部不同行都是仅差结局名的行 ⇒ 判「候选 ①」；有任一行有别的字段不同 ⇒ 判「候选 ②」；0 行不同 ⇒ 判「前提不成立」（第十条 RF1）；仅差结局名的行数另报，不进判定 | 问题一：「有没有一行的数值字段不同」，有 ⇒ ② |
| R2 | 行数与收尾行 | 新产物 `wc -l`、末行 `E7RESULT name=done emitted=N` 的 N，与留存的 3523 比 | 行数不等或 N 不等 ⇒ 候选 ②（行序缺了或多了）；N 必须等于新产物的行数（`replay.sh` 完整性闸 2），不等 ⇒ 作废（RV3） | 问题一 |
| R3 | 九个几何的 X3 行 | 九条 `name=x3_reserve_fit` 行逐几何整行比（含 `kappa=`、`largest_actual_empty_publish_slots=`、`code_reserve=`、`clause_reserve=`、`code_empty_publishes_held=`、`clause_empty_publishes_held=`） | 九行全等 ⇒ 对「n代、n条、κ、c实」这四样判「页上的数照旧」；任一几何任一字段不等 ⇒ 判「变了」，列几何与字段（门槛是 0 个字段不同） | 问题二：页上引的 X3 的数（第四节第 2 条）是否相同 |
| R4 | G-min 的选择 | 三条 `name=g_min slots_per_region=… chosen=…` 与对应的 `name=g_min_trial` 各行 | 三个 S 选中的档与页上的 `s4_g_min_64mib`、`s8_g_min_64mib`、`s16_g_min_128mib` 相同 ⇒ 「照旧」；任一不同 ⇒ 判「G-min 变了」并列出（几何标签变了第二段的几何常量表就对不上，R7 随之变） | 问题二：页上引的 n代 的 3、3、4 是不是还是这三个几何 |
| R5 | X5 的 `excess_slots` | 九条 `name=x5_overwrite_fixed_point` 行整行比 | 页上的四个几何（`s8_g_min_64mib`、`s16_g_min_128mib`、`s16_g_1`、`s16_g_4`）仍是 `excess_slots=2`、其余五个仍不是 ⇒ 「照旧」；否则「变了」并列 | 问题二：页上引的 X5 一句 |
| R6 | S = 16、G-1 的 n代 = 21 那一行 | `s16_g_1` 的 `name=x3_reserve_fit` 行整行比（页上把它当「n代 = 21，比一次挂载累计推空 22 次少」的第一段一半；「22 次」那一半来自第二段 Q1-3，不在这一次重测） | 与留存同一行逐字相同 ⇒ 「照旧」；不同 ⇒ 「变了」并列字段。它是 R3 的子集，单列只为问题二逐个数核时有一行对应页上那一句 | 问题二：页上的 n代 = 21 |
| R7 | 第二段输入表 | 对九个几何各取 (S、c实 最大、κ) 三个数（取自 X3 行，与 `scripts/experiment-apparatus/e173_admission_ceiling_bad_ring_slot_dispositions.rs` 第 3383 行起那张「第一段量过的 9 个几何」常量表逐几何逐列比；表本身不改、不起第二段） | 九个几何三列都相同 ⇒ 判「第二段输入没变，第二段的数照旧（推的，第二段装置不读 `crates/`，这一次不重测）」；任一不同 ⇒ 判「第二段输入变了，第二段的数要另登记重跑」 | 问题二：读法 A 下第二段那半 |
| R8 | 防线 | `grep -c 'name=void\|name=stop\|geometry_void'` 与 `grep -c NOT_RUN`，在装置标准输出与新产物上各数一次 | 合计必须 0；非 0 ⇒ 第十一节停机/作废（S2–S4、V1、V3、V6），不进 R1–R7 的「照旧」 | 无（防线） |
| R9 | 同输入两遍 | 同一份输入再跑第二遍，两份产物（去三个 `strip_` 之后）逐字节比 | 逐字节相同 ⇒ 记「确定」；不同 ⇒ 作废（RV5） | 无（作废条件；r1 第二段「确定性模型」同口径） |

**问题一与问题二的合判**（够判条件）：
- 问题一够判 ⇔ R1、R2 都得出（新产物完整、比对脚本过自检）。判定 = 候选 ① 或 ② 或「前提不成立」。
- 问题二够判 ⇔ 问题一够判，且 R3–R7 都得出。候选 ① 的条件 ⇔ R3、R4、R5、R6、R7 全部「照旧」（五个量，各自报，不合成一个数）；候选 ② ⇔ 其中任一「变了」，报哪几个几何、哪几个字段。注意：若问题一判 ①，R3–R7 所比的行不含结局名这一类字段（`x3_reserve_fit`、`x5_…` 行里没有 `NoSpace`），它们在构造上会等于留存；报告仍要把 R3–R7 各自读出来，不拿问题一的 ① 直接推。

## 七、钉绝对值的断言

装置不改，钉绝对值的锚点同 r1 第七节第一段部分（A1–A9、B10、B11），由装置自己的 `check_the_anchors`（第 2182 行）在每次开跑头一步判。分两类（照 r1 第十节 F3、第十一节 V3 的分法）：

- **出自被测条款本身的（甲类：A1–A3、A6、A9）**：不符不作废，走「条款可能错」：停下，把条款与装置算出的并列交回，条款与代码都去核（第十条 RF3）。
- **独立算出并用命令核过的（乙类：B10、B11）**：不符装置错，作废（`name=void clause=v3_anchor_differs`，第十一节 V3）。B10 钉的是 G-4 几何的单元区 [50176, 262144)；这一次只是原样再判，不新增、不改值。

这一次新增的、独立可核的绝对值只有两个，都是比对对象的性质，不是锚点：留存产物 `sha256sum` 与 `wc -l`（第四节第 3 条，已用命令核过）。比对脚本自己的绝对值断言在第九节：四份造出来的输入各得到什么分类。

## 八、轨迹与几何敏感性

- 被谓词消费的量：X3 的两个谓词是 n代 与 n条 对 3、7 的比，输入是 κ 与 c实 最大；κ 是准入谓词的输入（保留池 P代 = κ）。r1 第八节要求报 κ 在整条历史上的峰值、比第一个文件版本那一次的 κ 大的发布次数、期末值：这些在产物的每次发布一行（`name=publish…`）与几何一行里已经有，这一次不新增打印，**整份逐行比（R1）就把整条轨迹比了**，不只比端点；读数里报 R1 的不同行落在第几个发布。
- 几何：同 r1 第八节的取样点，9 个几何：S ∈ {4, 8, 16} × 池大小 {G-min, G-1, G-4}。方向相反的取样点：G-min 与 G-4（κ ÷ c实 的两头）、S = 4 与 S = 16（格式范围两端）。R3、R5 逐几何报，不聚合；「候选 ①」那一句若只在某几个几何上成立，报哪几个，不写成这条驱动的性质。
- 判别力：R1 的分类门槛（仅差结局名 vs 别的字段）靠第九节四份造出来的输入证它分得出，不靠这次观测到的 2 行变化本身。

## 九、变异

装置源码不改，没有新的装置变异；r1 的 MR1–MR8（`crates/mutations.tsv`，第一段）不在这一次重跑，提交时由门禁 `crates-mutation-replay` 照常判。这一次要新写的只有一小段比对脚本（执行员写，放 `research/results/e173-checkpoint-cost-against-empty-publish-cost-2026-11-13-r2-compare.py`，`python3 -I` 直接读两份文件、只读、不起装置），它的判别力用造出来的四份输入加三条弄坏写法证，每条写明它在哪个输入上改变输出：

| 编号 | 造的输入（拿留存产物改） | 预期 | 弄坏写法与它在哪个输入上改变输出 |
|---|---|---|---|
| C1 | 留存产物与它自己 | 0 行不同，分类「前提不成立」 | 弄坏写法「0 行不同也判 ①」：在 C1 上由「前提不成立」变「①」 |
| C2 | 留存产物里含 `NoSpaceAfterRaisingTheFloor` 的行，把那个名整段换成 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，其余逐字不动 | 全部不同行都是仅差结局名的行 ⇒ ① | 弄坏写法「结局名不换占位符直接比整行」：在 C2 上由「①」变「②」 |
| C3 | 在 C2 的基础上，再把一条 `name=x3_reserve_fit` 行的 `kappa=` 的数加 1 | 分类 ②，列字段 `kappa` | 弄坏写法「所有数字段都当结局名处理」（占位符范围扩到全部含数字的段）：在 C3 上由「②」变「①」 |
| C4 | 在 C2 的基础上，删掉最后一条非收尾行（行数少 1，收尾行 N 不改） | 分类 ②（R2 行数不等） | 弄坏写法「不比行数」：在 C4 上由「②」变「①」 |

四份输入都在草稿目录里造，不碰 `research/results/` 里的留存产物（只读）；脚本跑完四份输入的分类与上面一致、三条弄坏写法在它们各自那份输入上翻面，才算比对脚本过自检；不过自检 ⇒ R1、R2 不出（按没跑报）。

## 十、失败条款

| 编号 | 条款 | 什么观测会让它触发 |
|---|---|---|
| RF1 | 前提不成立：问题一的前提是「留存与今天的现跑有差」；现跑在这条驱动上与留存逐字相同，说明任务 13487 报的「4 行不同」在这一次的跑法上没复现，要么那一次的环境与这一次不同，要么差来自别的行（复跑表别的行、或另一份产物）。不当成「照旧」结案，停下并列两份证据交回主 agent | R1 得 0 行不同，且 R9 判「确定」 |
| RF2 | 问题一的候选 ② 成立：除了结局名还有数值字段变了。不是装置错，是结果；实验页第 1 次重跑一节引的第一段数要改，R3–R7 点名哪几个；第二段若 R7 判「输入变了」，第二段（`replay.sh` 第 2922 行）要另登记重跑，本登记不跑它 | R1 的任一不同行有别的字段不同，或 R2 行数/N 不等 |
| RF3 | 条款可能错：甲类锚点（A1–A3、A6、A9，第七节）与装置今天算出的不符 ⇒ 不作废，停下把条款与实测并列交回，条款与代码都去核（同 r1 F3） | `name=stop` 或锚点行里甲类项与 kb 条款不符；或某次取得出的空发布 c实 大于那一次之前的 κ（A6） |
| RF4 | 变化只落在某几个几何而不是全部：几何敏感（r1 F7）⇒ 记「不稳定」，两点都报，不写成这条驱动的性质 | R3、R5 在 G-min 与 G-4、或 S = 4 与 S = 16 上一边照旧一边变了 |
| RF5 | 阴性结果要证明路径跑过：R1 报「仅差结局名」时，要同时报被拒那一步在今天的代码上确实走到了新名所在的分支（新产物里含 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 的行数 > 0）；为 0 ⇒ 判「只差结局名」不成立，写「没跑到」，不写「名字换了」 | R1 判 ① 而 `grep -c NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 在新产物里 = 0（例：2 行变了、两个都不是新名）|

**跑之前写下的反向接受**：要是 R1 判 ①（2 行、只差结局名）、R3–R7 全部照旧，我接受「E173 第 1 次重跑第一段的数在今天的代码上不变，实验页引的那些第一段数不用改，只有产物里被拒结局的名字换了」，问题二判 ①，并加一句：第二段的数在读法 A 下是「输入没变所以照旧」，推的、没测；要是 R1 判 ① 但 R5 或 R4 变了，这一句不写，只写变的那几个。三句都按第六节的量写，不另找理由。

## 十一、作废条款与停机条款

**作废**（这一次的产物不算数，修完重跑）：

- V1 阳性对照在任何一个取样几何上读不出种下的效应（`name=void clause=v1_positive_control_routes`）。
- V3 乙类锚点（B10、B11）不符（`name=void clause=v3_anchor_differs`）。
- V6 某个几何上取得出的空发布少于 6 次（同 r1 V6）：那个几何作废，不当 0。
- RV1 比对脚本过不了第九节的自检（C1–C4 的分类不对，或三条弄坏写法在各自输入上没翻面）。
- RV2 一次运行经内存包装退 250–254（撞上限）：那一次的输出不算；不往上加内存，由任务服务放大重排。
- RV3 产物不以 `E7RESULT name=done emitted=N` 收尾，或 N 不等于产物行数（`replay.sh` 完整性闸 2），或退出码非 0（退 3 是停机/作废，照对应一条办）。
- RV4 一段跑的中途 `crates/` 变了：跑前跑后各取一次 `find crates Cargo.toml Cargo.lock .cargo -type f | sort | xargs sha256sum | sha256sum`，两次不等。
- RV5 同一份输入跑两遍，两份产物（去 `strip_` 三个函数处理之后）不逐字节相同（隐藏状态）。
- RV6 装置源码在开跑前后 `sha256sum` 与第四节第 4 条的 `a4cef951…` 不等（装置被改过，这一份登记就不是它的登记）。
- RV7 装置标准输出与新产物里 `grep -c NOT_RUN` 合计不是 0（装置不打这类行，是防线）。

**停机：装置与 `crates/` 对不上**（`implementation-first.md` 第 4 条：两边都查，既不作废也不当结果，停在这一格交回主 agent）：

| 编号 | 对不上什么 | 停下之后报什么 |
|---|---|---|
| S2 | 本地照 D28（挂载期承诺量） 已定项 4 的 Σ 名单重算的 κ 与核心层 `checkpoint_cost_of_the_version_to_build_on` 在某一次发布上不同 | 哪个几何、哪一次发布、各项本地值与差在哪一项（同 r1 S2；同时看 RF3） |
| S3 | 两条路数出的新占固定点在某一次发布、某块盘上不同 | 哪一次、哪块盘、两条路各多少 |
| S4 | 本地常量与 `crates/` 同名常量回比不等 | 哪个常量、两边各多少（今天的 `crates/` 改过，这一条最可能先触发） |
| S6 | 开工时 `crates/singlefs-core/src/mounted_session.rs`、`mount.rs`、`admission.rs`、`allocator.rs` 的 `sha256sum` 与第四节第 4 条不同 | 不停：这几个正是被测的输入，只把新哈希写进第十二节修订再往下做；若是装置源码（RV6）变了才停 |

**够判停机**：问题一、问题二两行在这一条驱动跑完、比对脚本出 R1–R9 之后各自够判（够判条件见第六节末）。这一次只有一段，没有「够判后未跑」的量；没有重跑的复跑表别的行，下面写明为什么：

- 第 2918、2919、2920 行（`checks`、`first`、`second` 三个模式，装置是 `scripts/experiment-apparatus/e173_admission_ceiling_bad_ring_slot_dispositions.rs`）：独立手写的计数模型，不读 `crates/`、不与实现共用代码；问题单两行问的是入库装置在今天的 `crates/` 上的输出，这三行答不了也不受 `crates/` 变化影响。该源文件在 git 暂存区里有改动（`git status --short` 显示 `M `），本设计员没比对改动内容；改动是否影响三份留存产物由复跑表那三行的准入按单元键现判（键相同即复用），不在这一次。
- 第 2922 行（`checkpoint-cost-reserve`）：同上是独立手写模型；它读第一段产物（`SEGMENT_ONE_PRODUCT`，`e173_admission_ceiling_bad_ring_slot_dispositions.rs` 第 3375 行写死留存产物路径）。第一段新产物落到新名字之后，留存产物仍在 `research/results/` 里不动，所以第 2922 行读的输入没变。只有 R7 判「输入变了」时才需要另登记重跑它，并把 `SEGMENT_ONE_PRODUCT` 指到新名字；这一次不改它。

**这一次要与之比的上一次产物**（比对对象表，门禁 doc-experiments 的 replay-comparison-units 格解析这张表）：

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e173-checkpoint-cost-against-empty-publish-cost-2026-11-02.out` | `85bed2435162a532566e64a1fc73bcaf4ab1f8c6a26b1b7a03c76135c89b07d9` | 3523 | `replay.sh:2921`（驱动 `driver_e173_checkpoint_cost`，exact，行项 `E173.4`）；原件已入库（提交 `4964b76b8`），工作区在 `research/results/` | `e173-checkpoint-cost-against-empty-publish-cost-2026-11-13-r2.out` | 比的是 R1 的差：留存与新产物逐行比，只比不要求相同；旧产物不覆盖、不改名 |

**复跑表那一行怎么换**（执行员做，在新产物落盘、R1–R9 出完之后；不在之前）：`scripts/experiment-apparatus/replay.sh` 第 2921 行第 4 列由 `e173-checkpoint-cost-against-empty-publish-cost-2026-11-02.out` 换成 `e173-checkpoint-cost-against-empty-publish-cost-2026-11-13-r2.out`，别的列不动（`E173|@driver_e173_checkpoint_cost||<新名>|exact`）；驱动上方第 4281–4283 行那段注释补一句指到本登记 `research/prompts/e173-r2-prereg.md`；驱动本身（第 4284–4286 行）不改。新产物是 `replay.sh` 在 `REPLAY_OUT` 下留的那份原始输出（含准入头）整份复制，不手改一个字符。旧产物留在 `research/results/`，下一次提交由归档那一步处理，本登记不删。机读表 `.claude/gate.d/replay-comparison-units.tsv`：E173 现在一行都没有；这一行要登记就要在 `crates/singlefs-implementation-experiments/src/bin/e173_checkpoint_cost_against_empty_publish_cost/main.rs` 里有一个单元表的单元名（该装置没有单元表，`formal_run_units` 这一类数组不在它里面），没有就照该表文件头写一行「不登记的比对对象」并挂一个开着的欠账号：**怎么登记待主 agent 定**（第一页「要主 agent 认的项」第 2 条）。登记之前 replay-comparison-units 格对这一行判红，执行员的报告里写明。

## 十二、修订

（装置写之后、产物之前由执行员写，只许收严或补臂。第十一节 S6 的新哈希写进这里。）

**修订 1（写于 2026-11-13，比对脚本写完之后、新产物一次没跑之前；原判据原样保留，只收严与补写）**

1. 脚本位置：第九节写的 `research/results/e173-checkpoint-cost-against-empty-publish-cost-2026-11-13-r2-compare.py` 改放 `scripts/experiment-apparatus/e173_r2_compare.py`（依据：派发提示「比对脚本放 scripts/experiment-apparatus/ 下」，README 已加一行）。判据与四份输入的写法不变。
2. 收严（R1）：`replay.sh` 的 `strip_timing` 把 `min=`、`max=`、`ratio=`、`dev=` 等字段值抹成 `X`，而本产物的 `name=x5_overwrite_fixed_point` 行就带 `max=`、`min=`，登记那一套（去准入头、负载、计时）比不出这两个字段的变化。R1 因此两套各比一遍：`r1_registered`（登记原套）与 `r1_unmasked`（只去准入头与负载字段）；任一套判 ②、或行数与收尾 N 不等，合判 ②；两套都判 ① 才判 ①；0 行不同才判前提不成立。R2 至 R9 不变。依据：读 `replay.sh` 第 3042–3044 行的 `strip_timing` 与留存产物 x5 行，不是单测读数（单测一次没跑）。
3. 补写：「别的字段不同」的行按字段名列出；「仅差结局名」的行另列新旧两边的结局段原文（`outcome:旧->新`）。按第一节「结局名」定义，`end=stopped_at:<阶段>:<名>:<名>` 整个值是一段，被一起换成占位符，阶段名变了从分类上看不出来，要从这一列读。
4. 弄坏写法：第九节表里是四行（C1–C4 各一条），派发提示写「三条」，按表做四条（脚本里的 `BREAKS`：`zero-diff-as-1`、`no-placeholder`、`numeric-as-placeholder`、`no-row-count`）；另加 R3–R9 的路径自证若干项，不改任何判据。
5. R3 取的行：九个几何各三种 `kappa_of_the_geometry` 共 27 条 `x3_reserve_fit` 行整行比（第六节 R3 写「九条」，九条是它的子集，按更严的比）。
6. 开工时的新哈希（S6）：这一段没有开工跑产物，不写，由承接的执行员读。

**修订 2（写于 2026-11-13，两遍产物与比对输出 15967 已出之后；只记两件与事实不符的写法，不动任何判据与读法；依据是主 agent 派发提示「关的是哪几条」第 1、2 条，不是单测读数）**

1. 产物文件名：第十三节与修订 1 写的「`replay.sh --item E173.4` 的 `E173.4.out`」与事实不符。`replay.sh --item E173.4` 把产物写成它的「本趟记号」`E173.1`，即 `<REPLAY_OUT>/E173.1.out`（两遍任务日志 15946、15948 的第 5 行「--item E173.4：本趟记号 E173.1」）。第一遍交比对时，主 agent 在两个产物目录里各把 `E173.1.out` 原样 `cp` 成 `E173.4.out`（`sha256sum` 与 `cmp` 相同：`d14b36500ecd3ba21833d373dcbd8128848f3c0619d17436d820ace1eae1255e`），比对 15967 读的是这份拷贝，内容与 `E173.1.out` 逐字节相同。比对脚本 `scripts/experiment-apparatus/e173_r2_compare.py` 现改成认 `E173.1.out`（自检补一格罩住，自检没在这一段跑过，主 agent 重跑）；15967 的 R1–R9 读数不受影响，因为输入逐字节相同。
2. 准入：第四节第 5 条与报告写的「`admission.py experiment … E173` 判准入」判不了 E173：准入任务 15940 退 2，报错原文「准入登记里没有实验键 E173（.claude/gate.d/stage-inputs.tsv）」。E173 这一行的准入实际由 `replay.sh` 按行判：两遍日志（15946、15948）里「E173 要现跑：键对不上（上一次 f93c77c85e30），变了 219 处：…」那一行，即输入变了、要现跑。第四节第 4 条的哈希是独立旁证，不变。

## 十三、读过的文件与跑过的命令

读过（文件加行号区间，grep 命中行也列）：

- `.claude/agent-common.md` 全文（开工先读；摘要读了「派发」「规则怎么读」「不做」里的「子 agent 不交任务」两条，行 1–117 一带）；`research/prompts/e173-r2-questions.md` 全文（34 行内）。
- `.claude/kb/experiments/173-上限卡在读坏槽的三种处置.md` 第 1–60 行、第 330–356 行（含第一段读数 X3 九行里前六行的数，第四节第 2 条已列；读了该页第 1 次重跑一节的现状句，其中有结论，已如实列在第四节）。其余各节没读。
- `.claude/kb/decisions/16-发布语义.md` 第 270–293 行（`quote-kb.py` 整段抄进第二节），第 276 行单独读了一遍；另 grep `已定项 10`、`射程`、`抬 F 之前先算` 命中第 42、54、113、270、276、287 行（只看了行首，没读内容）。
- r1 登记取出来的那份（`git show 59ce5a865:research/prompts/e173-r1-prereg.md` 存到草稿目录 `r1.md`，697 行）：第 1–34 行、第 218–372 行（大半）、第 505–604 行、第 664–697 行。
- `crates/singlefs-implementation-experiments/src/bin/e173_checkpoint_cost_against_empty_publish_cost/main.rs` 第 1–60、245–285、2300–2404、2405–2520 行，grep 命中第 251、252、854、997、1184、1241、1744、1897、2088、2182、2284、2304、2315、2365、2412、2427、2437–2491 行。
- `crates/singlefs-core/src/mounted_session.rs` grep 命中第 7、260、655–662、666、674、693、723 行。
- `scripts/experiment-apparatus/replay.sh` 第 1–40、2915–2923、4270–4330 行，grep 命中第 2184、2524、2614、5212、5216、5225 行；`.claude/gate.d/replay-comparison-units.tsv` 第 1–25 行；`scripts/gate/admission.py` 第 1–80 行；`research/prompts/e168-r1-prereg.md` 第 732–770 行、`research/prompts/e165-r7-prereg.md` 第 1–30 行（只看了重跑登记与比对对象表的写法）；`scripts/experiment-apparatus/e173_admission_ceiling_bad_ring_slot_dispositions.rs` grep 命中第 3375、3383、3880 行；`.claude/kb/checks-owed.md` 第 426 行；`.claude/gate.d/path-moves.tsv` 第 26 行；`.claude/heavy-thread-default`。
- 没读：`research/results/` 下任何产物的内容（只对留存产物取了 `sha256sum` 与 `wc -l`、`ls -l`、`git ls-files`）、`.claude/kb/experiments/` 下别的实验页、`records/`。全仓搜条款与用词没做（用不上）。

跑过的命令（设计员自己跑的，只读）：

```text
ls research/prompts | grep e173
git log --all --name-only --format= -- 'research/prompts/e173*' | sort -u
git show 59ce5a865:research/prompts/e173-r1-prereg.md   # 存草稿目录 r1.md
sha256sum research/results/e173-checkpoint-cost-against-empty-publish-cost-2026-11-02.out; wc -l < 同一份
git ls-files --error-unmatch 同一份; git log --oneline -1 -- 同一份
sha256sum crates/singlefs-verification/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs crates/singlefs-core/src/{mounted_session,mount,admission,allocator}.rs
python3 scripts/kb/quote-kb.py <草稿目录>/quote.md '.claude/kb/decisions/16-发布语义.md:270-293'   # 回读一致，退 0
bash scripts/hosts/work-directory-ownership.sh claim <草稿目录> --purpose 'E173 第 2 次重跑登记的草稿'
git status --short scripts/experiment-apparatus; git diff HEAD --stat -- crates | tail -3
```

要主 agent 跑的命令（设计员没跑，写进报告同名一节）：

```text
# 准入（第四节第 5 条）
task-server/task submit -- python3 scripts/gate/admission.py experiment /home/fy5090/code/singlefs E173
# 看行项（E173.4 就是第 2921 行；列表不编、不跑）
task-server/task submit -- bash scripts/experiment-apparatus/replay.sh --list-items
# 第一遍（R1–R8 的产物）与第二遍（R9）
task-server/task submit -- env REPLAY_OUT=<执行员的草稿目录>/e173-r2-run1 bash scripts/experiment-apparatus/replay.sh --item E173.4
task-server/task submit -- env REPLAY_OUT=<执行员的草稿目录>/e173-r2-run2 bash scripts/experiment-apparatus/replay.sh --item E173.4
```

`replay.sh --item E173.4` 跑完，行结果是「对不上，N 行不同 → diff <新> <留存>」或「一致」；两种都不代表登记判定，判定看 R1–R9。

## 十四、并行与挂钟

独立单位：这一次只有一条驱动（`replay.sh` 第 2921 行一行）；装置进程内 9 个几何（S × 池大小）与每个 S 的 G-min 五档试跑互相独立（装置按工作队列分到线程上，几何内部的历史是一串接着上一版的发布，只能按次序）；第一遍与第二遍（R9）互相独立，可以并排起。
线程：取登记值 `.claude/heavy-thread-default`（此刻 31，由任务服务套 `capped.sh` 给，登记不写包装）；装置进程内线程数 = min(31, 几何数 21)，同 `replay.sh` 第 4281 行一带注释（`threads_of_the_geometries`，config 行里与线程数相关的就这一个字段，产物里 config 行的 `threads_of_the_geometries=` 取 21）。
串行：比对脚本要等两份产物都出了才能比；复跑表那一行的换名要等 R1–R9 出完；没有计时类的量，不涉及计时串行；其余都能并行（两遍互相独立，R3–R7 读同一份新产物）。
预计挂钟：每遍约 25–70 分钟（推的、没量过：冷编 singlefs-verification 的 release 约 15–40 分钟，r1 第十四节给 checker 档包同类冷编 15–30 分钟；跑第一段约 10–30 分钟，r1 第十四节依据「约 3200 次发布加至多 15 次 G-min 试跑」）；两遍并排起则总挂钟约等于一遍加排队；比对脚本几秒；四份自检输入几秒。每遍都在 2 小时内，所以不写分片、续跑、多机三样：不需要；读数由承接的执行员填进这一行（第 3a 步 ①，量过的读数）。
预计在用核数：编译约 31 个核（31 线程）；跑装置约 21 个核（几何数 21 个，线程数 min(31, 21)）；比对脚本 1 个核。

