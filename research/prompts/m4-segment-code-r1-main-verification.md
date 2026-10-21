# m4-segment-code-r1 判决：里程碑四预开发整段代码轮第一轮（主 agent，2026-10-16）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 S1 S2 S3 S4 S5 S6 S7 S8 S9 G1 G2 G3 G4 G5 G6 G7 G8 G9 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 M4 V-a K-c -->

被判的产品代码（整段合入，冻结副本 `~/.local/share/singlefs/work/m4-segment-code-r1/repo/`，sha256 在 `m4-segment-code-r1-snapshot/crates-sha256.txt` 21 行）：`crates/singlefs-core/src/admission.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/block_device.rs`、`crates/singlefs-core/src/code_two_tree.rs`、`crates/singlefs-core/src/compaction_plan.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-core/src/logged_operations.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/records.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/region_scan.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/unlink.rs`、`crates/singlefs-core/src/write_accounting.rs`、`crates/singlefs-format/src/lib.rs`、`crates/singlefs-reference-filesystem/src/lib.rs`、`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs`、`crates/singlefs-replay-prefix/src/lib.rs`。正文 `research/prompts/_m4-segment-code-r1-body.md`，背景材料 `_m4-segment-code-r1-background.md`。

## 一、核查

- 快照：两条腿各自在冻结副本根上 `sha256sum -c` 21 行全 OK（两份报告开头原样）；主 agent 写判决前在工作树与冻结副本上各核一遍 crates 21 行、kb 快照一遍，全 OK。腿跑着的时候主 agent 没改被判文件（变异轻型重跑只读源码）。
- cite-check（以工作树为根，冻结副本只拷了 crates，kb 与判决路径要在工作树根下认）：正推 `✓ 核了 110 处引文、25 处裸行号，对上 110 处，没判 25 处（其中裸行号没有引文可比 25 处；逐处列在上面）`；攻方 `✓ 核了 31 处引文、5 处裸行号，对上 31 处，没判 5 处（其中裸行号没有引文可比 5 处；逐处列在上面）`。
- 复跑（各在新的草稿目录里，经 `run-with-memory-cap.sh 8G`）：攻方 `rerun.sh` → `name=rerun_total same=7 differ=0`（`~/.local/share/singlefs/work/m4-segment-code-r1-main-rerun-attack.log`）；正推 `rerun.sh` → `name=sonnet_forward_rerun_matches_expected result=same`（`…-main-rerun-forward.log`）。
- 主 agent 在工作树现查的行：
  - `mount.rs:8187` `central_mapping_tree: current.central_mapping_tree.clone(),`——T5 根因的锚点在。
  - `grep -rn 'SnapshotLifetime' crates/singlefs-core/src` 只命中寿命判定的枚举与它的两个臂（`transaction.rs:4843–4844`、`:4994`），没有强制开关——T2 缺项成立。
  - `compaction_plan.rs:902` `span_slots: 1,`——T9 对拍输入域缺口的锚点在。
  - `mounted_session.rs:1390` `| UserChange::CreateSnapshot => {`——T1 空文件集合拒的那一臂在。
  - `recovery.rs:2968` 注释「盘上读来的坏条目当损坏处置，可写挂载拒」；D8（核心索引结构） 已定项 5 定案句「**提示类**（整理）可暂停、可在停机谓词下提前收、交给盘那条线上不续做，不挡挂载」（`.claude/kb/decisions/08-核心索引结构.md` 第 108 行）。

## 二、判定与去向

| 格 | 判定 | 依据（腿 + 主 agent 现查） | 去向 |
|---|---|---|---|
| T1 建快照 | 与 S1、S2、「建快照算非空」一致；各类发布原样续快照条目（正推探针 s09）；零轮读法「事务号取上一版 + 1」相容。**规则没说**：文件集合为空时拒建快照 | 正推 T1；攻方没打中（一次抽样） | 用户 2026-10-16 定「放开」→ F5 |
| T2 释放分流与 deadlist | 与 S3 一致；**缺一项**：S3 末句「只供测试的开关：强制立即释放 / 进 deadlist」没有 | 正推 T2 第 5 条；主 agent grep 坐实 | F4 |
| T3 删快照 | 一致（批大小共用开关、销毁中的快照建拒读照读回退拒、续做到空） | 正推 T3；攻方没打中 | 无 |
| T4 回退拒与读快照 | 核心层的读者对码 14 / 15、8、12 放行（正推探针）；**打中（判据一侧）**：无快照时回退、再建快照、再在快照下覆盖写，池级 checker 在这段合法历史上红 I-3.5（种子下标 2022 缩到 5 步，量的）；S8「第一版拒回退，不出现」的前提不成立。根因按代码推：I-3.5 的「在哪棵用户可见树里」取自跨全部版本、按落点查的一张表（`crates/singlefs-checker/src/walk.rs:1525`） | 攻方 T4，复跑 `i35-seed2022.txt` 相同 | F9（池级 checker，测试代码）；S8 的 (b) 款读法补「回退之后建快照」一形，归阶段同步写回 |
| T5 一批搬迁 | 映射为准、钉住单元搬走后删快照放的是目的地（正推）；**打中（最重）**：整理搬过单元之后回退，回退那一版的中央映射照抄现行版、指向回退自己释放掉的目的落点，之后覆盖写被拒 `ReleaseTargetAlreadyReleased`（模型答该成），I-3.9、I-3.15 每一步红，抬 F 之后再回退多红 I-3.11；16 格中 6 格打中，崩不崩都中（量的）。随机历史没撞到：整理那一组比重不抽回退（`crates/singlefs-harness/src/history.rs:1176`）。另：`publish_relocation_batch` 的注释与行为不符（正推）；多出来的重写是否越过 `ckpt_cost` 没造出取样点 | 攻方 T5，复跑 `remount-sweep.txt`、`seed126-prefix-suffixes.txt`、`rollback-minimal-trace.txt` 相同；主 agent 现查 `mount.rs:8187` | F1 |
| T6 意图上盘与续做 | 字段表与各发布对上；**不一致（正推，量的）**：签发那次发布开新聚簇段时 `segs0` 写 3310、记账第 11 项 3309，与 G1 定义句差一；**打中（攻方，量的，后果轻）**：R 被同一次发布本来就要重写的节点腾空时那一批不删意图，多一次只删意图的发布、模型判不合（种子下标 1055 缩到 13 步）；水位 V-a 恒 0（`compaction_plan.rs:257`，已知自报） | 两腿；复跑 `compaction-seed1055-trace.txt` 与正推 s12 相同 | F2、F3 |
| T7 R 排他与合取 7 | R 排他四条路无漏口、`empty_segments` 增量与全扫 32000 次比较全等（正推探针 s01）；**后果（正推，读码推的）**：产品路径上整理不起跑（批大小产品值 `NotDecidedByTheClauses`、三条水位恒 0），合取 7 的 N₀ = 1 开着，一块盘没有全空聚簇段时前台写入类发布全拒、不推抬 F | 正推 T7；攻方没打中新的 | 用户 2026-10-16 定「维持，记后果」：不改；批大小与水位的产品值等 E10（老化：结构抗老化 vs 事后整理） 给数再定 |
| T8 discard 调用者 | 一致；`crates/mutations.tsv` 第 4453 行的改法在三条 harness 用例上不超时就红（正推）；**打中（攻方，按代码推的，后果轻）**：因孤儿头扣下、读失败、报错没发的段之后不补发；没造出 discard 打到仍被引用的段 | 两腿 | 不改代码：discard 是可选的回收动作，少发不破坏一致性；记一行欠账（阶段同步时取号写进 `checks-owed.md`） |
| T9 区域扫描与计划修订、Verus 核 | 第 2 批 T4、T5 的反例关掉、没新开口子；**缺口（正推，量的）**：核与核心层对拍的输入域记录跨度全是 1（`compaction_plan.rs:902`），把打分改成数记录条数（M4）对拍与 harness 全绿；本机无 verus，78 verified 两腿都没复核 | 正推 T9 | F8 |
| T10 恢复读侧 | I-2.5 提前后合法搬迁与快照镜像不被拒；坏 logged_ops 条目让可写挂载整个拒、只读与冷走读收下 | 正推 T10；攻方没打中 | 主 agent 定、不改：D8 已定项 5「提示类……不挡挂载」说的是续做不挡挂载，不管盘上条目字节坏了；坏条目照 `recovery.rs:2968` 当损坏处置。读法写进 D26（后台整理与放置回收） 已定项 2 的射程，归阶段同步（零轮） |
| T11 RefFS 与模型 | 三操作与「非空」口径同核心层；**打中（攻方，量的）**：变异第 4346 行有了会红的输入（两版各一个快照、活头相同、只差快照抄的根）；**口径不同（正推）**：RefFS 在读出的文件对上几个允许状态时取下标最大那个的快照集合（`crates/singlefs-reference-filesystem/src/lib.rs:584–594`），核心层两态都可能 | 两腿；复跑 `t11-original.txt`、`t11-mutated.txt` 相同 | F6、F7 |

零轮读法：正推逐条判 17 条全部相容（`segs0` 另差一，即 T6）；攻方逐条攻过的没有打中。两腿分歧：没有——T5、T4、T6 只有攻方打中，正推在同一格报「一致」是因为它核的是判决原文，打中的都在判决没写到的形状上（回退那一路、跨版本的标签表、腾空 R 的那一批）。

没打中的格（T1、T2、T3、T7、T9、T10）攻方只抽了一次，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」不记「没打中」，留给第二轮。

## 三、改法（都被攻过零轮，进修复批，第二轮攻它们）

- **F1**（`mount.rs` 造回退那一版的上一版，`version_to_publish_the_rollback_after`）：中央映射树取目标那一版的（与释放集、复活集同按目标版的映射算），照判决 G3「映射是唯一权威」；目标版带整理意图时 R 排他照挂载恢复那一路恢复。验收：攻方 `compaction_then_remount_then_rollback_sweep` 16 格全 `Completed`、`seed_126_prefix_then_suffixes` 各后缀 I-3.9 / I-3.15 / I-3.11 不红；`history.rs` 整理那一组比重加上回退（测试代码）；变异：把取法改回照抄现行版，点名的用例红。
- **F2**（`transaction.rs` / `mounted_session.rs` 整理一批）：这一批发布之后 R 按记录全空的（含计划里的单元都是这次发布本来就重写的节点），同一次发布删意图（G4「目标态到达与最后一批同发布删」）；`batches_published` 不把没搬单元的发布记成一批。验收：攻方种子下标 1055 那 13 步与模型逐版对上；变异一行。
- **F3**（签发那次发布的 `segs0`）：照 G1 字段表定义句取值，签发那次发布开新聚簇段时与记账第 11 项一致。验收：正推探针 s12 两格同值；变异一行。
- **F4**（S3 末句）：加只供测试的开关「强制立即释放 / 强制进 deadlist」，两臂各一条用例强制进入（`.claude/rules/fs-design.md`「每条分支必须能被测试强制进入」）；变异各一行。
- **F5**（用户 2026-10-16 定「放开」）：现行那一版有树而文件集合为空时建快照放行——`mounted_session.rs:1390` 那一臂、RefFS 的 `CreateSnapshot`、harness 模型三处一起改；用例：空文件集合上建快照、再写文件、读快照读到空。
- **F6**（RefFS 崩溃语义）：读出的文件对上几个允许状态时，快照集合按全部对上的状态取允许集合，不取下标最大那一个；用例：写了之后报错再崩溃的建快照与删快照，两种结局都收。
- **F7**（harness 模型单测，测试代码）：攻方 T11 那条用例（或同形）进 `model.rs` 测试模块，变异表第 4346 行点名它。
- **F8**（`compaction_plan.rs` 对拍单测，测试代码）：输入域加跨度 2 的记录、段内多于 2 条活记录；M4（数记录条数不数槽数）要红，留一行变异。
- **F9**（池级 checker `crates/singlefs-checker/src/walk.rs`，测试代码）：I-3.5 的用户可见树标签按版本记（每一版走读时自己记），不取跨版本的表；用例：攻方种子下标 2022 的 5 步历史（无快照回退 → 建快照 → 快照下覆盖写）I-3.5 不红；另一形（种子下标 1032，标签非 None、(b) 款「整棵继承」落空）是不是同一根因，实现员先核再定，不同就报。
- **F10**：变异没红四行（12、409、2058、4436）照调查员 `~/.local/share/singlefs/work/m4-no-red-investigate/report.md` 的改法补；变异轻型停时没判清的六行（72、623、4296、4479、4463、4492）卡空出来后点名重跑，红的并进同一批。

## 四、交给下一批的实现员（按文件聚簇）

| 簇 | 改法 | 文件 |
|---|---|---|
| 甲 核心：回退、整理一批、`segs0`、强制开关、空池快照的核心一侧 | F1、F2、F3、F4、F5（核心）、F8 | `crates/singlefs-core/src/mount.rs`、`transaction.rs`、`mounted_session.rs`、`compaction_plan.rs`；harness 的回退 / 整理 / 快照用例与 `history.rs` 比重 |
| 乙 RefFS 与模型 | F5（RefFS 与模型）、F6、F7 | `crates/singlefs-reference-filesystem/src/lib.rs`、`crates/singlefs-harness/src/model.rs` 与 RefFS 翻译 |
| 丙 池级 checker | F9 | `crates/singlefs-checker/src/walk.rs` 与 checker 的用例 |
| 丁 变异没红 | F10 | 照调查员报告点名的文件，与甲乙丙不相交的单派，相交的并进那一簇 |

甲与乙对空池快照要同口径：乙的派发写甲的读法（放行、读快照读到空），不等甲交回。

## 五、复跑与核不动的

- 两腿复跑见第一节。
- 核不动的：Verus 78 verified 两腿都没复核（腿的机器上没有 verus），以形式化轻型在 m4-predev 上那一趟绿为准；T5「多出来的重写越不越过 `ckpt_cost`」与「零个全空段」「合法镜像触发 `TwoRecordsMovableUnderOneMappingKey`」三个取样点两腿都没造出，留第二轮。
- RefFS 的允许集合与核心层崩溃语义的对照攻方没打（报告「没做什么」），正推量了一处（T11）。

## 六、交用户（已定）

| # | 题 | 用户定案（2026-10-16 弹窗） |
|---|---|---|
| 1 | 产品路径上整理不起跑而合取 7 开着 | 「维持，记后果」 |
| 2 | 文件集合为空时建快照 | 「放开」 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 2 | 支撑 | 2026-10-16 不受影响：F1、F2、F3 照它已定的事务形态与字段表改实现；T10 的读法（坏条目当损坏）归阶段同步写进射程 |
| D26（后台整理与放置回收） 已定项 1 | 不影响 | 2026-10-16 不受影响：用户定维持合取 7，批大小与水位的产品值等 E10（老化：结构抗老化 vs 事后整理） |
| D5（快照 / 空间记账机制） 已定项 14 | 不影响 | 2026-10-16 不受影响：F9 改的是池级 checker 取标签的实现，引用区间的条件不动 |
| D8（核心索引结构） 已定项 5 | 不影响 | 2026-10-16 不受影响：T10 按「提示类不挡挂载」说的是续做读，条款不改 |
| D16（发布语义） | 不影响 | 2026-10-16 不受影响：建快照事务号取上一版 + 1 两腿判相容，F5 只放开空文件集合 |
