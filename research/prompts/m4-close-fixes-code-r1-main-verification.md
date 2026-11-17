# m4-close-fixes-code-r1 判决：代码轮 r3 之后那两批修补，第一轮（2026-11-09）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

被判的文件（开工快照 `research/prompts/m4-close-fixes-code-r1-snapshot/crates-sha256.txt`）：`crates/singlefs-core/src/compaction_plan.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`。

## 一、核查

- 腿：正推 `research/prompts/m4-close-fixes-code-r1-sonnet-output.md`，攻方 `research/prompts/m4-close-fixes-code-r1-opus-output.md`。
- 两腿都全程只抽一次；「没打中」的格（T4、只试一次）不支撑结论。
- 攻方模型 `research/prompts/m4-close-fixes-code-r1-opus-model/` 复跑（任务 10447）判决写下时还没出结局，模型结论都算推的；下面判「打中」的都是主 agent 现读代码核过的：
  - `crates/singlefs-core/src/transaction.rs:5217`–`:5221`：`MappingTreeNodeBelowTheRoot`、`MappingTree`、`TreeTable`、`InstanceTable`、`InstanceTablePageAfterTheFirst` 记 `false`，同文件 `:11739` 的注释写「它们的旧落点照释放之前读盘核那一路走」，两处相反。
  - `crates/singlefs-core/src/mount.rs:5380`：`instance_table_chain_of_root(&*devices, &current.root)`，盘表读者不计。
  - `crates/singlefs-core/src/transaction.rs:11905`–`:11913`：`NodeMalformedOrNoEntryWithThisKey`、`EntryNarrowerThanItsFieldTable` 记 false，一份真的只坏一份改记没坏。
- 用户原话（`.claude/kb/decisions-history.md` D4（校验和位置） 节 2026-11-05 块）：「冷恢复读、可写挂载时建上一版那一读、可写挂载期间别的读读到只坏一份要不要搬，选「可写挂载时搬，冷恢复只计数」」。

## 二、逐格判

| 格 | 判 |
|---|---|
| D4（校验和位置） 已定项 4「可写挂载期间别的读」 | 打中（两腿一致）。用户读文件（会话没有读入口）、推抬 F 读实例表（`mount.rs:5380`）、释放前读盘核（`transaction.rs:5352` 起）、搬留存读源（`transaction.rs:11182`）都不计不搬 |
| D4 第 73 行三类豁免 | 打中。映射树叶挂载与头一次改动都不碰（`code_two_tree.rs:301`），没有路径主动修；三类旧落点不走释放前读盘核。那半句是第一批实现员「推的」写回 kb 的 |
| `transaction.rs:11909` 一判 | 打中（轻）：提示过期走不到；映射里没有这把 key 只有坏镜像走得到，少计一次；`recovery.rs:417`、`mounted_read.rs:1123` 同形 |
| T3 | 打中名字与文档：`relocation_of_the_old_data_units_kept_for_the_snapshots` 与 `mounted_session.rs:205`、`:219` 的文档只说快照，现在也报挂载记下的那一批 |
| T4、只试一次 | 没打中（各抽一次，不支撑结论） |
| 两种计数口径 | 数上不打架，理由句打架（`mounted_session.rs:261`、`:263`，`mounted_read.rs:997`），改注释 |
| C946（只坏一份的读没有计数也没有搬迁修复） | kb 过期：正文与验收还是旧说法，验收要冷恢复与只读挂载搬，与 D4 相反 |
| 检查单：走不到的理由 | 不成立：两份实现员报告都没有这一节；派发提示以后点名 |
| 检查单：类型与名字 | 不成立：`inode_record_is_not_below_the_separator_key_and_the_container` 三个裸 u64 同形；上一行 T3 那个名字 |
| 检查单：不兼容、判定库、多机、多线程 | 成立 |
| 检查单：带 --item | 不成立：第一批报告命令是占位且引已不存在的脚本；项以 `crates/mutations.tsv` 现行为准 |
| 检查单：新崩溃流 | 推不出，算打中：GPU 只读挂载打开新加的下界判法（`crates/singlefs-verification/src/crash_judge_recovery.wgsl:286`）没有流也没有变异行 |

## 三、用户定案

用户 2026-11-09 弹窗：「别的读」选「按字面全接上」；三类豁免选「现在补主动修复」。主 agent 推荐的是收窄条款与登欠账，用户没选。

## 四、交实现员（一批）

1. 可写挂载期间四处读都计数并记下要搬的：会话加读入口，用户读经它走；读实例表、释放前读盘核、搬留存读源接上计数。
2. 三类（映射树节点、树表、实例表片）读到只坏一份时的主动修复：下一次用户改动发成之后另发的那一批里换落点，只试一次，被空间拒照推抬 F 再试一遍，结局随那次改动报出（与 D4 已定项 4 对数据单元的定案同形）。
3. `transaction.rs:11909`、`recovery.rs:417`、`mounted_read.rs:1123` 映射里查不到时不把真的一份改记没坏，配坏镜像用例。
4. T3 成员改名与文档、`:11739` 注释、两种口径的理由句、`inode_record_is_not_below_the_separator_key_and_the_container` 三个 u64 换 newtype。
5. GPU 只读挂载打开下界判法补流与变异行。

## 五、后续

实现改完之后这一题开第二轮代码轮，攻第四节的改动。kb 写回（D4 第 71、73 行，C946）随那一批收尾。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D4（校验和位置） 已定项 4 | 不影响 | 2026-11-09 不受影响：条款字面不改，改的是实现；三类那半句与 C946 随实现写回 |
