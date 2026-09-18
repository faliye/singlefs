# 里程碑「第二个事务」步 0 / 步 3 那批代码三方对抗第一轮：主 agent 核实（2026-09-17）

<!-- doc-lint:not-numbers S1 S2 S3 S4 S5 S6 S7 S8 S9 S10 S11 X1 X2 X3 X4 X5 X6 X7 X8 X9 X10 X11 F1 F24 -->

材料：正文 `research/prompts/_m2-step3-code-r1-body.md`（S1–S11 选择表、X1–X11 判据、四条腿的分工）、背景材料 `_m2-step3-code-r1-background.md`（正文 + 14 份 kb 文件的小节清单 + 附录一 31 段整抄）、附录二 `_m2-step3-code-r1-diff.md`（从提交 5f9e449 起 `crates/` 的全部 diff + `mount.rs` 与步 3 用例全文 + `crates/mutations.tsv`）。
四条腿：云端攻方（Opus，`m2-step3-code-r1-opus-output.md` 263 行，模型 `m2-step3-code-r1-opus-model/`）、云端正推（Sonnet，`m2-step3-code-r1-sonnet-output.md` 480 行）、本地攻方（`m2-step3-code-r1-local-attack-output-s2.md`、`-s3.md` 两份干净样本，1164 / 800 词；s1 空、void1 / void2 作废）、本地辩方（`m2-step3-code-r1-local-defense-output-s1.md` 一份干净样本 1090 词，五次里四次判红：只有一份，按「一条腿只抽一次样不算一次观测」记「样本不够」）。核查员 `m2-step3-code-r1-verifier-output.md`（第七节）。

被判的文件（门禁 56 号按路径认）：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/unit.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/image.rs`、`crates/singlefs-harness/src/crash.rs`；用例 `crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs`、`second_transaction_step_zero_layer0.rs`、`checker_known_bad_images.rs`、`common/mod.rs`；这一轮的改法落在 `crates/singlefs-core/src/recovery.rs`（第五节）。

## 一、本地攻方（X3、X5、X7、X11；两份样本，每问先看两份一不一致）

| 问 | s2 | s3 | 主 agent 核 |
|---|---|---|---|
| 1（X3 分配器重建） | 1a NO HIT（算术复述）；1b HIT：「建文件 → 删文件 → 重开，已释放而无根引用的单元仍在已分配里、checker 并集不含 ⇒ I-3.1 红」；1c 指出 `STATISTIC_INODE_WATERMARK` 每次发布写死 `FIRST_INODE_NUMBER + 1`；1d / 1e NO HIT | 1a NO HIT；1b HIT（同一构造）；1c 同；1d NO HIT | 1b 两份一致，但构造用了「删文件」：第一版没有删除，释放只发生在被换下那一刻（D3（空间分配） 已定项 7），换下它的那条根仍在环里、仍引用它，并集里有它。**没打中**；它说的机理正是回退与抬 F 之后要面对的（被抛弃根、F 之下的根引用的单元），步 4 / 步 5 的代码按实例表与 F 收候选集，另立一轮判。1c 两份一致：今天一个 inode 恒真，多 inode（并行线三）要改成从 inode 树取，记进并行线三的决策点 |
| 2（X5 暖机） | 2a HIT：盘数 > 3 时上限 3 次盖不住全部盘、`mount_writable` 照样返回 Ok；2b NO HIT；2c HIT：暖机不核 checkpoint 保留池；2d / 2e NO HIT | 2a 同（区域映射全落一块盘）；2b NO HIT；2c 同；2d NO HIT | 2a 两份一致：第一版两块盘（D2（RAID 条带策略） 已定项 9）、`ROOT_RING_REGIONS = 3`，盘数 > 3 是第一版之外的形态；上限 3 次盖不住时今天静默返回——记进步 3 决策点「暖机上限与盘数」。2c 两份一致：准入不等式与保留池今天都没实现（C283（准入失败时不先推发布就报 ENOSPC） 那一格），不是这一步新开的洞。**都没打中这一轮的判据（触发观测是「返回之后一块盘上没有本实例的根」，两块盘上不会发生）** |
| 3（X7 I-3.8 射程） | 3a：根落了、实例表单元没落 ⇒ 判到旧表；3b：实例表单元装满会 panic；3c：I-7.7 与 I-3.8 互相矛盾 | 3a 同；3b 同；3c 同 | 3a：单元写在根 FUA 之前一道屏障（D16（发布语义） 已定项 7），层 0 的状态里根落了单元必在，「根落了单元没落」不是可达状态——没打中。3b：第一版 88 字节一行、一片 32 KiB 装 372 行，链指针记录（C304（实例表链指针记录装不下 83 宽的指针） 收口）是给第二片留的、第二片今天没写者——记进决策点（射程）。3c：超级块 5、根 6 ⇒ I-7.7（超级块实例代号不低于根环） 红是对的、I-3.8（实例表行唯一且低于挂载根） 按根判也是对的，两条各判各的——没打中 |
| 4（X11 第二轮七处改法） | 4a NO HIT；4b：空闲计数变负会 panic；4c / 4d / 4e NO HIT | 4a–4e 全 NO HIT | 4b：`free_slots` 只在 `mark_allocated` 里减，而它先断言跨度里的槽都空，减不到负——没打中。七处改法两份都没打中；4c、4d 两份一致地把 oracle 那一臂与 Ignore 那一遍的计数核了一遍 |

小结：本地攻方零打中；三条接缝记进决策点（inode 水位写死、暖机上限与盘数、实例表一片装满）。

## 二、本地辩方（只有一份样本）

六处取法各写了最强的反对再答，其中两处值得记：
- 取法①「实例表当第九个角色、写在八个 bump 角色之后」：辩方腿按代码指出这句只对内存里的 `units` 查找向量成立；主 agent 现读 `crates/singlefs-core/src/transaction.rs` `written_units`（按 `rewritten` 的次序取）与 `rewritten_roles()`：写行那次发布的单元写次序是实例表、分配记录、记账、映射、树表——**实例表写在最前**；发号次序也是实例表 0、树表 1。里程碑步 3 现状那句改写（第五节）。
- 取法⑤「所选根自己那条记录读不出时从水位之上同实例最小的一条接」：辩方腿判它站不住，与云端攻方的 X1 同一个结论（第四节）。
其余四处（暖机次数现算、开放段不续、没有干净关闭标记、树表空拒开）辩方判站得住，理由与正文一致；只有一份样本，不记「没打中」。
顺带：本地腿的字词损坏闸把提示里的 Rust 路径 `::` 判成粘连（两条本地腿各撞一次），提示改成散文写法才过；这是闸的假阳性，记进 `.claude/kb/tooling.md` 之前先在下一轮提示里避开。

## 三、云端正推（S1–S11、X9、X10）

| 格 | 判 | 主 agent 核 |
|---|---|---|
| S1 | 一致；「所选根自己那条读不出时从最小的一条接」条款没说 | 核实（全仓零命中）；这一格与 X1 合并处置 |
| S2、S4、S6、S7、S8、S9、S10、S11 | 一致 | 逐格对了附录原文与代码行号，判对 |
| S3 | 一致；「开放段不续」「重开后第一次分配从空闲位图重找」条款没说 | 核实：里程碑现状已标预想 |
| S5 | 一致；jsn 取「全环最大 + 1」比「前缀末 + 1」宽 | 核实：两者在第一版只差在有撕掉记录的时候，与 X1 同一格 |
| X9 | E142（第一个事务的干跑） 装置与 `crates/` 的两条恢复规则逐行相同 | 判对；这一轮 X1 的改法要同步进装置（第五节） |
| X10 | 里程碑步 0 / 步 3 现状、登记表两行、I-3.8 状态列与代码对得上；取号那一行的屏障归属已按代码改 | 判对；步 3 现状里「实例表写在八个之后」那句按第二节改 |

## 四、云端攻方（X1、X2、X4、X6、X8）

| 格 | 报告 | 主 agent 核 | 处置 |
|---|---|---|---|
| X1 | **打中**：所选根自己那条记录读不出时 `expected_next = None`，连续性检查整段不执行，水位之上最小的那条无条件接上；副本上改坏 txg 5 / 6 的根槽与 jsn 4、5 两份镜像，jsn 6 被直接接上、读回第四版 | 核实：`recovery.rs` 那两行逐字如报告所引；在入库装置上重做那段历史（`torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg`，改法之前跑它 `prefix_applied = 1`、读回第四版）。故障模型是「两份镜像各改一字节」、不是 D13（验证路线） 已定项 4 的整写子集——收：仓里 `first_transaction_step_six_recovery.rs` 与步 3 的链首用例用的是同一个模型，它们算数的理由同样适用于这一格；层 0 够不着它是因为记录先于根持久，根在而自己那条记录撕了只有介质损坏才造得出 | 改法：所选根自己那条记录读不出时，链首只认 `checkpoint_txg = 根的 txg + 1` 的那条（第一版一次发布一条记录、txg 每次加一，这一条与「锚点 jsn + 1」在可达状态上等价；一次发布多条记录时要连同第六条一起改）。保守读法（没锚点一条都不施加）会把 `torn_journal_record_truncates_the_prefix_but_recovery_still_completes` 打红——那条用例是一次单点损坏（jsn 2 撕了、根槽坏了）之后一条完好的记录仍该救回来的形态，不该放弃。改法被攻过零轮，进步 4 / 步 5 代码那一轮的攻击面 |
| X2 | 打中（弱）：① 只 mkfs 的池可写挂载报 `NoPublishedVersion`；② 释放判定路径对盘上读回的两条位置条目 `assert_eq!` 而不是报错；③ `own_record` 的回退可能取到别的实例的记录（今天不承重） | ① 核实，射程没有条款，代码不动、记决策点；② 核实是读代码读出来的（没造出镜像），第一版从盘上重建的位置条目来自校验和核过的单元、两盘同槽由写者保证，改成 `Result` 是收严、记进欠账；③ 核实不承重：`mount_writable` 交出去的 `current` 是真发布的输出 | ① 记进里程碑步 3 决策点；② 记 C（第六节）；③ 不动 |
| X4 | **打中**：同一段历史写出的行 (1, 6, 4)，事务 3 没被施加而 W 把它罩住 | 核实（入库装置上同一条用例：改法之前行是 (1, 6, 4)）；它是 X1 的下游 | X1 的改法之后行是 (1, 4, 0)（jsn 4、5 都撕）或 (1, 6, 4)（只撕 jsn 4、jsn 5 接上、施加两条）——后者 W = 4 罩的正是被施加的前缀，用例两格都钉住 |
| X6 | 没打中（五种形状）；判别力观测：jsn 全池接着数只被一句 `assert_eq!(…counter, 5)` 钉着 | 核实 | 不动 |
| X8 | 没打中：五个对照搬到到 C 的脚本上逐格同形红；偏移 24 / 28 读对；重开接缝合成 4 写一段核过；判别力观测：把 24..28 改成 20..24 到 C 的两条非 ignored 用例全绿、记录核对器对手摆的三个正常终态也判 `claimed_missing` | 核实 | 到 C 的脚本上的对照与偏移判别力记进步 6 的对照清单 |

## 五、处置

1. **代码**（`crates/singlefs-core/src/recovery.rs` `replay_journal`）：所选根自己那条记录读不出时链首只认 `checkpoint_txg = 根的 txg + 1` 的那条，之后照旧 jsn 连续。会红的用例 `torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg`（`crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs`：撕 jsn 4、5 ⇒ 一条不施加、读第二版、行 (1, 4, 0)；只撕 jsn 4 ⇒ 接上 jsn 5、6、读第四版、行 (1, 6, 4)）；变异 `crates/mutations.tsv`「链首没锚点时不看 txg、无条件接上水位之上最小的一条」它红。原来那条「链首不接在所选根自己那条记录之后」的变异在改法之后成了等价变异（锚点丢了照样按 txg 停），换成「链首锚点错一位」，由 `stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row` 判红。E142（第一个事务的干跑） 装置同步同一条规则（第一条流上产物不变：根在而自己那条记录撕了不是层 0 的状态）。
2. **正文订正**：判据表把前缀口径写成「五条」，kb 原文是「六条，缺一不可」（第六条：施加的单位是一次发布，第一版一次发布一条记录时空转）；`18-块里携带什么信息.md` 已定项 11 行记录字段名「所选根的 checkpoint_txg」与写行规则「重放之后那个根的 checkpoint_txg」说的不是同一个量，代码跟的是后者——记进决策点。
3. **里程碑步 3 现状**：「实例表在 bump 序列末尾当第九个角色」改成「实例表是第九个角色，写行那次发布它的单元写在最前、发号也在最前（`rewritten_roles()`）」；加「所选根自己那条记录读不出时链首认 txg + 1 那条」。
4. **决策点（交用户）**：见第六节。

## 六、交用户的决策点（都标预想、代码按最保守可行的读法）

| 项 | 今天的取法 | 谁打的 |
|---|---|---|
| 所选根自己那条记录读不出时链首接哪条 | `checkpoint_txg = 根 txg + 1` 的那条；一次发布多条记录时要连同第六条改 | 云端攻方 X1、本地辩方⑤ |
| 第一版可写挂载的射程 | 只接在已发过文件版本的池后面；只 mkfs、或第一次可写挂载后没发文件就关掉的池报 `NoPublishedVersion` | 云端攻方 X2 ① |
| 释放判定路径对盘上读回的位置条目 | `assert_eq!` 两盘同槽；改 `Result` 是收严 | 云端攻方 X2 ② |
| 暖机上限与盘数 | 上限 `ROOT_RING_REGIONS` = 3 次，盘数 > 3 时盖不住也返回 | 本地攻方 2a |
| 实例表一片装满 | 372 行之后没有第二片的写者 | 本地攻方 3b |
| inode 水位统计量 | 每次发布写 `FIRST_INODE_NUMBER + 1` | 本地攻方 1c（并行线三） |
| D18 已定项 11 行记录字段名与写行规则的量 | 代码按「重放之后那个根的 checkpoint_txg」 | 云端攻方顺带 |

## 七、核查员（`m2-step3-code-r1-verifier-output.md`，286 行）

- 判别力自证：把一条 Opus 引用的行号 +1 再核，判 ✗，核法分得出对错行号。
- Opus 报告的两条复跑命令在核查员自己的副本上跑出的末行与报告逐字一致（`attack_opus_r1.rs` 的 sha256 一致）；X1 那段历史在入库装置上另有一条用例重做（第五节）。
- 核了 96 处引用与事实：✓ 79、✗ 10、核不动或部分核 6。Opus 报告 30 处引用里 2 处行号偏 1–2 行（内容对）；Sonnet 报告在 S2 与 X9 两段集中 6 处行号错（一处 `recovery.rs:422` 实际在 510 行），S1 / S3–S8 / S11 准确；本地两条腿提示里的事实段抽查一致（辩方 10 / 10）。
- 核查员另报一处观测：E142（第一个事务的干跑） 装置在「所选根自己那条记录读不出」那一支比 `crates/` 多一个 `checkpoint_txg = 根 txg + 1` 的分支。主 agent 核：那是这一轮 X1 的改法先落到了装置（同一天第十次跑），`crates/` 的同一改法随步 4 / 步 5 的代码一起并回主树，两边今天又是同一条规则（`第五节 1`）。
