# m4-close-fixes-code-r2 判决：r1 第四节那批修补，第二轮（2026-11-09）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

被判的文件（开工快照 `research/prompts/m4-close-fixes-code-r2-snapshot/crates-sha256.txt`）：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/compaction_plan.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`。

## 一、核查

- 腿：辩方 `research/prompts/m4-close-fixes-code-r2-sonnet-output.md`，攻方 `research/prompts/m4-close-fixes-code-r2-opus-output.md`。两腿各抽一次，「没打中」的格不支撑结论。
- `python3 scripts/agents/cite-check.py` 两份：`✓ 核了 98 处引文、44 处裸行号，对上 98 处，没判 44 处`；裸行号没逐处核，判决引到的行主 agent 现读。
- 写判决时 `sha256sum -c` 快照 7 份都 OK；判决引的 `mount.rs:4256`、`:4801` 现读过，内容与腿引的相同，`:10728`、`:10847` 只过了 cite-check。
- 攻方模型复跑任务 11051：`name=attack_fixes_r2_one_copy_target rc=0 expected=0`，`test result: ok. 8 passed`，观测在 `~/.local/share/singlefs/work/m4-close-fixes-code-r2-opus/rerun-1/observations.txt`。下面引的读数都是这一趟的。
- 模型是腿的副本装置，读数不进 kb；要进 kb 的由这一批的入库用例重做。

## 二、逐格判

| 格 | 判 |
|---|---|
| D4（校验和位置） 已定项 4 与用户定案「按字面全接上」 | 打中（两腿一致）。没接的读：`mount.rs:4801`（`attack-fixes-r2: ceiling changes_before_the_damage=0 outcome=NoSpaceAfterRaisingTheFloor … other_after_the_change=1 … table_moved=false`，F 抬不动时只有这一读）；这批自己新加的认位置那一读（`mounted_session.rs:2163` 经 `transaction.rs:11966`，`repair_chain_read outcome=NotNeeded other_after_the_change=1 counted_by_a_counting_reader=1 … table_moved=false`）；`mount.rs:4256`、`:10728`、`:10847`；`instance_table_sweep.rs:356`、`:741`。攻方写明没穷举 `region_scan` 与整理读 |
| 映射 key 被换、封得对的盘 | 分歧已核：辩方判够不着（只测过提示没过期那一形）；攻方模型 `key_replaced_after_relocation mapped_is_old=true mapped_is_new=false mount_count=1 violated_after_the_mount=["I-1.11", "I-2.1", "I-3.11", "I-4.8", "I-7.4", "I-8.10", "I-3.15", "I-3.17"]`。提示过期时把映射盖回隔离的旧落点，打中。`recovery.rs:3607` 重建只数条目数。只读路径两腿都读码判走不到错读，没量 |
| 七条 ① 没读到的坏副本 | 代码没打中；D4 第 73 行那半句与实现反着，kb 写回改 |
| 七条 ② 攒批读不到待发 | 没打中，攻方只查了 D16，条款空白，记进第五节 |
| 七条 ③ 账的口径 | 打中（轻）。`other_reads_after_the_mount data_mount=1 data_read_again=1 data_relocated_without_another_read=1 leaf_mount=1 leaf_read_again=0 leaf_replaced=0`：同是挂载读到一次，数据单元与映射树叶之后的计数不一样 |
| 七条 ④ 读盘核之后才被拒 | 读码走得通，没有用例；补用例 |
| 七条 ⑤ 提前返回丢账 | 打中（轻）：`transaction.rs:13403`、`:13413` 两个 `?` 走不到 `:13467` 的记账 |
| 七条 ⑥ | 没打中 |
| 七条 ⑦ 实例表读不出整批拒 | 打中（轻）：`unreadable_instance_table outcome=RefusedNotForSpace refusal=InstanceTableOfTheVersionUnreadable(UnitUnreadable { slot: SlotNumber(50304) }) unit_kept_still_at_the_damaged_location=true`，无关的组把好 key 连带丢掉 |
| 只试一次 | 打中（轻）：`tried_again other=3 outcomes=[…RefusedNotForSpace…×3]`，再读一次就再试一次，文档「只试一次」写宽了。搬两次两腿都没找到 |
| 放回字节的判法 | 打中：树表那条「不再引用」断言没判别力（树表每次发布都换）；放回之后 checker 抓不抓得住产品写错，表上没有一行证过。任务 10799 换的那行红在 `transaction.rs:13627` 的产品断言上，任务 10801 那行证的是反方向。攻方模型给了更硬的判法：`unrestored data_unit_relocated violated=[]`、`tree_table_replaced`、`instance_table_replaced_by_the_mount`、`mapping_tree_leaf_rewritten` 都零违例，对照 `control_data_unit_not_repaired` 报 I-2.1、I-4.8、I-7.4 |
| 坏盘 × 部分落盘 | 打中：短段子集只在放回字节的基线上判 checker；`recover` 只跑前缀 |
| 名字与死链 | 打中：`relocations_of_the_units_read_with_one_copy_failing_while_mounting`（`transaction.rs:12123`）现在也收别的读；`transaction.rs:12116` 文档死链；三个 `…KeptForTheSnapshots…` 开关名 |
| 走不到 | 不成立一处：`transaction.rs:11953` `TreeTable` 那一支攻方判走不到，实现员没报；实现员查实，走不到就删 |
| GPU 下界 | 打中：`crash_judge_recovery.wgsl:663` 那一处没有变异行（现读过，同句 `lt64(inode, separator)`）；GPU 那一行点全量域，基线在共用卡上超时（任务 10661 基线日志，`crash_judge_dispatch.rs:1524`） |
| 测试文档引的变异行 | 打中：`unit_read_…rs:584`、`:1243` 点名的行在表里没有（主 agent 删重复行时留下的，现读过） |

## 三、用户定案

用户 2026-11-09 弹窗：映射 key 被换的盘选「挂载时拒」。「别的读」沿用 r1 判决第三节用户定的「按字面全接上」，不再问。

## 四、交实现员（一批）

1. 可写挂载期间同形的读全部接上计数与记账：`mount.rs:4801`、认位置那一读（读到的片同一批接着换）、`mount.rs:4256`、`:10728`、`:10847`、`instance_table_sweep.rs:356`、`:741`，并把 `region_scan` 与整理读穷举一遍，报告列全。
2. 建上一版核「映射 key 集合 = 进映射单元的 key + 留存条目的 key」，不等就拒（内存里比，不多读盘）。对应改 `a_disk_with_a_key_replaced_in_the_mapping_is_mounted_and_the_mount_puts_the_key_back`、`a_read_back_with_no_entry_in_the_mapping_still_counts_the_damaged_copy`；崩溃判器走同一判的，GPU 判器与 CPU 判法同步。
3. 账：⑤ 提前返回也记账；⑦ 实例表读不出时豁免那半跳过、`planned` 照发，结局报明；③ 口径统一；「只试一次」的文档收窄成写实的那一句。
4. 名字与死链；`transaction.rs:11953` 那一支查实走不到就删。
5. 测试：树表断言换成「旧落点带隔离记号」；加「卸载之后不放回判真盘」的判法与「坏盘 × 部分落盘」；④ 补用例；第二份坏与读回 `None` 各一条；`wgsl:663` 补变异行；GPU 那一行点缩域的用例；`unit_read_…rs:584`、`:1243` 改对。

## 五、延后

- ② 攒批读不到待发写：条款空白，登 `.claude/kb/checks-owed.md`（kb 写回那一批）。
- kb 写回（D4 第 71、73 行，C946，映射 key 那一判）随这一批收尾。

## 六、下一轮

改完开第三轮代码轮，也是这一题的最终轮，攻第四节的改动；第四节第 2 条的拒判被攻过零轮。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D4（校验和位置） 已定项 4 | 不影响 | 2026-11-09 不受影响：条款字面不改，改的是实现；kb 写回随实现 |
| D21（权威态与派生态的分界） 已定项 10 | 不影响 | 2026-11-09 不受影响：映射 key 这一判照用户定案实现，判据不改 |
