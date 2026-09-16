# 发布 B（覆盖写 + 释放）那批代码三方对抗第一轮：主 agent 核实（2026-09-16）

材料：`research/prompts/_m2-code-r1-body.md`（正文）、`_m2-code-r1-checklist.md`（12 份清单）、`_m2-code-r1-appendix.md`（36 段，回读逐字节一致），拼成 `_m2-code-r1-background.md`（2329 行）；附录二 `_m2-code-r1-diff.md`（diff 与两个新测试全文，1485 行）；本地腿英文提示 `m2-code-r1-local.md` 自足。

被判的文件（门禁 56 号按路径认）：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-harness/src/crash.rs`；新测试 `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs`、`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`。

## 一、本地腿（s1 326 词、s2 314 词；字词损坏闸两次都过）

| 问 | s1 逐字要点 | s2 逐字要点 | 主 agent 核 |
|---|---|---|---|
| 1（释放的口径） | 「Could not construct such a history」：已释放的槽标成占着，分配器不发 | 「During checkpoint 5, the allocator reuses slot 50180」 | 两次相反。s2 假设了一条今天不存在的复用路径：`DeviceFreeMap::is_free` 对已释放的槽恒 false，没有任何代码把它放回去（步 5 才做）；它说的「读第 3 代的数据被第 5 代盖掉」也不是违例——第 3 代根出候选集之后 I-7.4（近 K 代块未被复用） 不护它。记「不稳定、没打中」 |
| 2（记账口径） | 崩在 B 的分配树写出之后、根之前：记账 23 槽对并集 13 槽 ⇒ I-3.1 红 | 「checkpoint 3's root is removed by checkpoint 8 … allocated bytes still counts them」 | s1 核不通：B 的记账节点在 B 的根持久之前不被任何有效根引用，checker 读的是最新有效根（A）下面的记账（13 槽），54 号两条流全量零违例是证据。s2 的数错（根环 24 槽不是 5、defer 窗口不是「10 个 checkpoint」），但方向对：A 的根被轮转覆写之后 checker 的并集不再含 A 独占的那 10 槽，而分配器的「占着」不缩 ⇒ I-3.1（已分配统计对得上） 会红。这正是 X2 问的第三种情形；今天的脚本走不到（要 24 次发布），步 5 的回收（释放代 ≤ max(F_生效, 环里最旧有效根) 就放回空闲）必须在根被覆写之前把它们放掉，或者记账要按「仍被有效根引用」维护。⇒ 记进里程碑步 5 的预想与决策点，不改这一轮的代码 |
| 3（恢复的放开） | 18 条记录（少于 20）会被接受 | 20 条里两条同槽，恢复接受 | s1 误读：`recovery.rs` 逐字「`< 10 * device_count` ⇒ 拒」。s2：走读不查 key 唯一，但池级 checker 的 `check_index_node_keys` 要求条目 key 严格递增，同 key 两条判红；走读自己没判是事实，属于 E142 走读同款那批「只判够第一个事务的」检查——不进这一轮 |
| 4（oracle 假阴性） | 构造不出 | 构造不出 | 两次一致没打中；理由与代码相符（点名验证不过 ⇒ 记录不施加 ⇒ 走 A 读第一次内容 ⇒ 与第 3 代版本相符） |
| 5（漏的检查） | 释放代写错、树表诞生 txg 改了、超级块世代没进、已释放标志没置 | 同 + 映射条目没删、反向链指错、提交标记没验 | 逐条对新用例：释放代 = 4、树表诞生 txg = 3、超级块世代 6、A 的映射 key 一条不留、反向链 = A 记录头的 CRC 都有断言（`second_transaction_step_one_overwrite.rs`）；提交标记在 `replay_journal` 里判（`!record.is_commit ⇒ break`）。没有一条是新的 |

本地腿小结：X2 的「A 的根被覆写之后并集缩而占着不缩」是这一轮唯一站得住的方向（s2 提出、主 agent 核实机理），不是今天代码的 bug、是步 5 要接的口径；X1 / X4 没打中；X5 全被既有断言罩住。

## 二、正推腿（Sonnet，`m2-code-r1-sonnet-output.md` 127 行）

报告第 3 行逐字：「结论先给：**S1–S7 全部一样，未发现代码与条款不符处**；X3（恢复放开）未打中；X6 有一处接缝要记欠账（BirthSequenceAllocator 每次发布重建），其余三处写死当前判无害；X7 现状文本与代码逐句核对一致，两处轻微措辞落差记在下面，不构成「现状与代码不符」。」——第 3 行的「X3 未打中」与第 111 行「**判定：X3 打中一半**（第 1 条未打中，第 2 条打中）」自相矛盾，按逐条正文（第 111 行）算，不按开头那句。

| 项 | 报告怎么说 | 主 agent 核 | 处置 |
|---|---|---|---|
| S1–S7 | 逐条「一样」，每条引了条款行号与代码行号；S2 用 `walk.rs` 的 `references: BTreeMap<(u32, u64, u64), String>` 与 `walk_root(..., false)` 遍历环里其余根坐实读法甲（第 36 行） | 抽核 S2 与 S6：`walk.rs` 那两处与报告说的一致；S6 说的三处放宽 / 不动逐字对得上 `recovery.rs` | 无 |
| X3 第 1 条（已释放记录代晚于根） | 未打中：走读判「不晚于根」 | 与代码一致（`record.generation > root.checkpoint_txg ⇒ 拒`） | 无 |
| X3 第 2 条 | 第 111 行逐字：「`recovery.rs:625-631` 的判据只检查 `roots.allocation.entries.len()`（全池合并总数）`>= 10*device_count` 且 `is_multiple_of(device_count)`，没有任何一处按设备身份分组核对「每盘条数相等」」；举例盘 0 有 20 条、盘 1 有 16 条的镜像会通过；并写明「这不是这次改动新引入的缺口……只是把同一处既有的盲区又放大了一圈」 | **打中，核实**：改动前判 `!= 10 * device_count`，同样只看总数；改动后任何 ≥ 20 且是盘数整数倍的总数都过。它是走读自检，不是池级 checker 的不变量（I-3.1 / I-5.1 走独立的引用集合），报告这一句也对 | **改代码**：`recovery.rs` 新增 `allocation_records_are_one_per_device`（逐盘取（槽, 跨度, 代, 已释放）集合、每盘不少于 10 个落点、各盘集合相同），走读改调它；五条单测（含「一盘改写成已释放、另一盘没有」也拒），其中 `placement_recorded_on_one_device_only_is_rejected_even_when_the_total_is_even` 造的正是「盘 0 十一条、盘 1 九条、总数仍 20」；把比较臂改回 `true`（等于旧判法）它红、改回绿。`check.sh` 全绿 |
| X6 `BirthSequenceAllocator` | 第 115 行逐字：「清零的时机是「换到下一个 checkpoint」，不是「每次调用一个装某个对象的函数」……多个对象在**同一个 checkpoint** 里各自调用一次类似 `publish_file_version` 的入口……两个对象各自的码 2/码 3 单元会在同一个 `(inode 树, 这个 txg, 这个实例)` 键上分别拿到从 0 起的出生序号，撞出重复」 | **核实**：`transaction.rs` 第 700 行 `let mut sequences = BirthSequenceAllocator::default();` 在 `publish_file_version` 顶部；D19（块指针的结构与宽度预算） 已定项 9 原文如报告引。今天一次调用 = 一个 checkpoint，字节不受影响 | **记决策点**：写进里程碑并行线三「会碰到的决策点」（开工前挪到 checkpoint 作用域）；不改这一轮的代码 |
| X6 其余（`FIRST_INODE_NUMBER` 写死、`placements` 按种类推跨度、`inode_object_birth` 传 txg 3） | 当前无害，后者留一句耦合观察 | 与代码一致 | 无 |
| X7 步 1 / 步 2 | 逐句一致；两处轻微措辞落差 | 读了两处：不改现状句子的意思 | 无 |
| X7 顺带 | 第 123 行：步 0「现状（2026-09-16 建档）：未开工」与已落地的 A→B 两次发布层 0 装置不符 | **核实**：`second_transaction_step_zero_layer0.rs` 与门禁 54 号第二条流都在 | **改 kb**：步 0 现状改成「部分开工」，列出已落地与未做的 |

正推腿小结：S1–S7 没有一条与条款不符；打中一处（逐盘核，改代码）、记一处（出生序号作用域，决策点）、顺带一处（步 0 措辞，改 kb）。

## 三、攻方腿（Opus，`m2-code-r1-opus-output.md` 233 行；副本装置上跑，数不入库）

| 格 | 报告怎么说（第几节） | 主 agent 核 | 处置 |
|---|---|---|---|
| X2 I-5.2 恒真 | 第二节：`allocator.rs` `free_slots()` 整个实现是 `self.unit_area_slots - self.allocated_slots`，`transaction.rs` 第 882 行注释写「第 2 项独立维护、不由「容量 − 已分配」现算」而第 885 行调的正是它；checker 判 `free + allocated == capacity`，两个几何常量同一个式子 ⇒ 「`allocated_slots` 取任何值 I-5.2 都绿」；变异 M-X2（已释放的从已分配里减掉）副本上 I-3.1 红、I-5.2 绿 | **打中，核实**：三处代码逐字如报告；D5（快照 / 空间记账机制） 已定项 4 的 ⚠️ 逐字「第 2 项必须独立维护，不许由 `容量 − 已分配` 现算」。改动前（HEAD `allocator.rs` 第 127–128 行）就是这么写的，第一个事务那一轮没人打中 | **改代码**：`DeviceFreeMap` 加独立的 `free_slots` 计数，`new` 置单元区、`mark_allocated` 减、`free_slots()` 直接返回；注释改成与代码相符；验收断言把空闲钉成绝对值 3 472 506 880 并另钉单元区 211 968。会红：把 `mark_allocated` 里的减法摘掉，`cold_start…` 用例里 checker 的 I-5.2 判 `Violated("盘 0：空闲 Some(3472883712) + 已分配 Some(376832) ≠ 单元区 Some(3472883712)")`——改动前这条变异 I-5.2 纹丝不动 |
| X1-① defer 只增不减 | 第三节：`deferred_slots` 只有 `+= span` 一处写、位图不清；可再分配谓词全仓只在注释里；副本 49 轮覆盖写空闲单调降 8 028 160 字节 | **核实**：与代码相符；它是步 5 的题面（D16（发布语义） 已定项 1 的谓词要 F_生效 与环里最旧有效根，两样都还没有），不是这一轮的 bug。报告要的是「不许留成静默的空白」 | **写明**：里程碑步 2 现状加一句「可再分配谓词没有实现、defer 只增不减、界没有上限」，指到 C283（准入失败时不先推发布就报 ENOSPC） 与 I-5.3（报出的空闲都兑现得了） 未实现那一格；不另立欠账（那一格已有） |
| X1-② 第 50 轮 panic | 第三节：分配记录每轮 +16，全部塞进一个 `build_index_node`，第 50 轮 820 条 × 20 > 16384 − 头 ⇒ `unit.rs` 的 `assert!` panic | **核实**：`unit.rs` 那条断言在；容量按头 135 算 812 条，20 + 16 × 49 = 804 过、820 不过，与副本读数一致 | **改代码**：`unit.rs` 公开 `index_node_entry_capacity`，`publish_file_version` 在动分配器之前算「这次之后的记录数」，超过就报 `PublishError::AllocationRecordsExceedOneNode { records, capacity }`；节点分裂留给步 3 之后。会红：用例 `repeated_overwrites_report_a_full_allocation_node_instead_of_panicking`（49 次成功、第 50 次 `Err { records: 820, capacity: 812 }`、最后一版没被释放），把判定改成恒假就 panic 在 `unit.rs` 的断言上 |
| X4 oracle 丢实例维 | 第四节：`PublishedVersion` 只有 txg，`newest_persisted_root_txg` 取 txg 的 max，`oracle_violation_for_versions` 第 563 行 `let Some((_, effective_txg))` 把实例丢掉；副本喂两条同 txg 不同实例的版本：落在低实例判 `None`、读出实例 2 的内容反判违例 | **核实**：三处逐字如报告；D22（单元原子性怎么合成） 已定项 7 逐字「择新在 checkpoint_txg 平局时按它高者赢」。B 这条流单实例、不可达，报告自己也说清了射程 | **改代码**：`PublishedVersion` 加 `instance`，`newest_persisted_root` 返回 (txg, 实例) 按字典序取 max，oracle 按 (txg, 实例) 比、按两者找版本；根记录的实例代号从根槽写的偏移 24 读。会红：两条单测（落在低实例必须判违例；同 txg 两实例是两条版本），改回只比 txg / 只按 txg 找各红一条。步 4 要的「被抛弃实例的根同时在环里」那一格记进步 4 决策点（oracle 要实例表当输入） |
| X5-① M1 / M3 只有验收断言 | 第五节：释放代写错、树表诞生 txg 改了各只红一条断言，三条独立路（checker、走读、层 0）都不响；里程碑步 2 那句「没有会红的检查」写宽了 | **核实**：`second_transaction_step_one_overwrite.rs` 第 201 / 276 行那两条断言在，都读内存态输出 | **补对照的形态记账**：立 C374（释放代与树表诞生 txg 只有验收断言盯着），里程碑那句收窄成「只有验收断言、没有会红的不变量」，立不立不变量步 3 开工前定 |
| X5-② 映射那一格 | 第五节：`mapping_entries` 每次发布从六个指针新建、「删掉 A 的六条」不是一条代码路径，那条断言是算术必然；更重的：释放入参是 `previous.placements()`（上一版内存态槽号），不经映射，D19（块指针的结构与宽度预算） 已定项 5 第 1 条逐字「释放一律经映射，不经提示」；步 2 验收那两条「释放判定路径报「不在映射」」没被测过而现状写「四条全过」 | **打中，核实**：`publish_overwrite` 改动前逐字 `&previous.placements()`；映射只在解引用的回落支路出现。「四条全过」这句写宽了 | **改代码**：新入口 `placements_to_release_via_mapping`——上一版八个单元里六个进映射的按各自的映射 key（`TransactionOutput` 新字段 `mapped_units`）在上一版映射节点里查落点，查不到报 `ReleaseNotInMapping`、一个落点都不释放；映射树与树表从根记录的两条指针取；`publish_overwrite` 改调它。会红：用例 `release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released`（经映射取的八个落点与写者记的槽号相同；B 之后 A 的码 1 key 在 B 的映射里查不到；把 A 的映射节点重装成少一条再覆盖写 ⇒ `Err(ReleaseNotInMapping { unit: Data })`、分配器一条记录没改），把释放改回按提示它红 |
| 顺带 ①（X6）分配器每次挂载从零重建、没有从分配记录树恢复的路径 | 第六节 | **核实**：`recovery.rs` 没有构造 `PoolAllocator` 的地方；第一版范围，D3（空间分配） 已定项 7 点名的那道闸实现是空的 | 步 3（第二个可写实例要重开）的题面，已在步 3 设想里；不另记 |
| 顺带 ②（X7）步 2 现状那句「空闲 = 单元区 − 已分配」与代码相符、`transaction.rs` 注释不符 | 第六节 | 与 X2 同一件事 | 随 X2 改：现状句改成独立维护，注释改成与代码相符 |

攻方腿小结：五格全打中、全核实；四处改代码各有一条会红的用例，一处写明空白，一处立欠账。副本上的数一个都没引进 kb：改代码之后的读数（812、804、820、3 472 506 880）都是入库装置上跑出来的。

## 四、判决

1. **本地腿**：X1 / X4 没打中（两次抽样各有一次相反、核不通）；X2 第二次抽样提出「A 的根被环覆写之后并集缩、占着不缩」，记进步 5 决策点。
2. **正推腿**：S1–S7 一样；X3 打中一半 ⇒ 改代码（逐盘核）；X6 一处 ⇒ 并行线三决策点；X7 顺带 ⇒ 步 0 现状。
3. **攻方腿**：五格全中 ⇒ 四处改代码（空闲独立维护、装不下报错、释放经映射、oracle 带实例）、一处写明、一处立 C374（释放代与树表诞生 txg 只有验收断言盯着）。
4. **改了的文件**：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/unit.rs`、`crates/singlefs-harness/src/crash.rs`；用例 `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs`（两条新用例 + 空闲钉绝对值）、`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`（版本带实例）；`recovery.rs` 与 `crash.rs` 各加一个单测模块。六条变异各自会红（逐盘核、空闲不减、装不下不判、释放按提示、oracle 只比 txg、oracle 只按 txg 找版本），做法与红在哪写在上面三张表里。
5. **按跑前的反向接受条款，X1 / X2 / X4 打中要再攻一轮**：第二轮攻方腿只攻这四处改法本身（`m2-code-r2-opus.md`），第二轮的判决另开文件。
6. **不入库的**：攻方腿副本上的全部读数与两个探针文件。

## 五、第二轮辩方腿复核之后的补记（2026-09-16）

辩方腿（`m2-code-r2-sonnet-output.md` 第四节）判：二十格技术判断没有一格判错；两格处置不合跑前条款——X1-①（defer 只增不减）与本地腿 X2 第二次抽样（A 的根被环覆写之后并集缩、占着不缩）都判了「打中」，条款要「改代码、补一条会红的用例、再攻一轮」，而第四节只写了字、指了既有欠账。核实：条款原文没有给「要等步 5 的子系统」开例外，第四节也没有明写偏离。

处置：

1. **明写偏离**：这两格是同一个病根（可再分配谓词没实现），谓词要的 F_生效 与环里最旧有效根都是步 5 的题面，在步 2 里实现它等于把步 5 提前做；里程碑的步序是用户定的，这一轮不越过。所以这两格按 X6 的待遇处理（记决策点 + 写明空白），偏离了 X1–X4 的条款，理由如上。
2. **补弱形态的会红用例**：`released_placements_are_not_handed_out_again_before_reclaim_exists`（`second_transaction_step_one_overwrite.rs`）把今天的形态钉住——三次发布之后每盘 defer 20 槽、占着 33 槽、空闲 211968 − 33，释放过的落点一个不再发、C 的数据单元落 50184。会红：把 `mark_released` 改成清位图（立即复用），它红在分配器「同一个落点释放了两次」那条断言上——被立即复用的槽再被释放时撞上旧记录。步 5 接上回收那天它也必须红（defer 会减、50180 会回来），到时按谓词改写。它不是 I-5.3（报出的空闲都兑现得了） 的检查，只是让「空白」在用例里有一个会变的读数。
3. 「再攻一轮」这一格在第二轮攻方腿（`m2-code-r2-opus.md` Y1–Y4）里没有单独的题，因为改法就是「不改」；第二轮判决记它为「明写偏离、弱形态用例、步 5 还」。
