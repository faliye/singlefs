# 发布 B 代码三方第二轮：辩方腿报告（Sonnet）

角色：复核第一轮判决（`research/prompts/m2-code-r1-main-verification.md`）每一格判得对不对，不攻代码本身。
方法：读第一轮三条腿原报告、跑前条款（`_m2-code-r1-body.md` 第三节）、改后代码全文与两个新测试（`_m2-code-r2-diff.md`）；
在 `/tmp/claude-1000/-home-fy5090-code-singlefs/e166d536-b665-4a88-8372-3091b9a0fc41/scratchpad/m2-code-r2-sonnet-copy/`
（`rsync -a --exclude target --exclude .git`）上逐条施加六处变异、跑对应测试、记录红在哪、还原；副本基线 `cargo test --workspace --no-fail-fast` 全绿（4+2 非 ignored 用例、1 条 ignored 全量用例未跑），六处变异跑完全部已还原（`grep -rn "MUTATION-" crates/` 命中 0）。
原仓 `/home/fy5090/code/singlefs` 全程只读，未改任何已有文件，未做任何 git 写操作。

## 一、逐格判——判决第一节（本地腿）

| 格（判决表行） | 判 | 依据 |
|---|---|---|
| 问 1（X1 释放口径，s1/s2 两次相反，判「不稳定、没打中」） | 判对 | `crates/singlefs-core/src/allocator.rs:160-168` `mark_released` 只有 `self.deferred_slots += span;`，`allocated[]` 位图一位不清；`lowest_user_data_slot`（`allocator.rs` 附近）只读 `allocated[]` 挑候选。s2「checkpoint 5 复用 slot 50180」这条历史今天没有任何代码路径能构造出来，判决「没有任何代码把它放回去（步 5 才做）」与源码相符 |
| 问 2（X2 记账口径，s2 第二次抽样「A 的根被环覆写之后并集缩、占着不缩」，判「不是今天的 bug、记进步 5 决策点」） | **处置不合条款**（技术判断本身准确） | 详见下方「特别核 (1)」 |
| 问 3（X3 恢复放开，18 条记录 s1 误读；s2 重复槽 s2 判「不进这一轮」） | 判对 | `recovery.rs` 判据字面是 `entries.len() < 10 * device_count \|\| !is_multiple_of(...)`（不是 `!=`），s1「不检查下界」不成立；`crates/singlefs-checker/src/walk.rs` 的 `check_index_node_keys`（现查 `walk.rs:17` 起）用 `key_of(pair[0]) >= key_of(pair[1])` 挡重复 key（`walk.rs` 第 187 行调用处），s2 的重复槽历史会被池级 checker 这条挡住，判决「走读自己没判是事实……不进这一轮」成立 |
| 问 4（X4 oracle 假阴性，两次一致「没打中」） | 判对 | 两次抽样方向一致；机理（未验证单元 ⇒ 记录不施加 ⇒ 走 A 读第一次内容）与 `recovery.rs` 的验证/施加逻辑相符，判决未再深入核，无异议 |
| 问 5（X5 漏的检查，判「没有一条是新的」） | 判对 | 副本上重跑六条变异（见文末总表），M1(释放代)/M3(树表诞生 txg)/M4(超级块世代)/M-X2(defer) 均如判决所说各红在既有断言上；s1/s2 列的「释放代写错、树表诞生 txg 改了、超级块世代号没进、已释放标志没置」四项确实都有既有断言盯着 |

## 二、逐格判——判决第二节（正推腿）

| 格 | 判 | 依据 |
|---|---|---|
| 开头「X3 未打中」与正文「X3 打中一半」自相矛盾，判决「按逐条正文（第 111 行）算」 | 判对 | 逐条正文（第 111 行）给出了具体机制与举例，第 3 行只是摘要句写错；按详细分析裁决是合理的证据取舍 |
| S1–S7「全部一样」 | 判对 | 抽核 S2：`crates/singlefs-checker/src/walk.rs:52` `references: BTreeMap<(u32, u64, u64), String>`、`walk_root` 两次调用在 `walk.rs:727`/`733`（分别 `true`/`false`），与报告「727-735」区间相符；S1（`allocator.rs:16` 起的标志位、`release` 不删条目）、S4/S5（`transaction.rs` 的 `FilePublish` 字段、`journal.rs` 反向链）逐项读码与报告描述一致 |
| X3 第 1 条未打中 | 判对 | 同「一、问 3」 |
| X3 第 2 条打中，改逐盘核 | 判对（副本验证） | 详见文末「六条变异」表第 1 条 |
| X6 `BirthSequenceAllocator` 每次发布重建，记决策点 | 判对 | `crates/singlefs-core/src/transaction.rs:777`（`publish_file_version` 起始）`let mut sequences = BirthSequenceAllocator::default();`；今天一次调用=一个 checkpoint，字节不受影响，机理与报告一致（报告行号 700 与现读 777 的落差是后续 diff 在它之前插入了代码，不影响结论） |
| X6 其余两处（`FIRST_INODE_NUMBER`、`placements()` 按种类推跨度）当前无害 | 判对 | 与攻方腿报告第三节末尾对同两处的独立复核结论一致（两条腿各自核实，互不依赖） |
| X7 步 1 / 步 2 现状与代码一致 | 判对 | 数字逐项见 `second_transaction_step_one_overwrite.rs` 断言，副本 `cargo test` 复核全绿 |
| X7 顺带：步 0「未开工」与已落地的层 0 装置不符，判「改 kb」 | 判对 | `.claude/kb/milestone/02-second-txn.md` 步 0 现状句已改写（现查含「部分开工」字样，见下方特别核 (1) 引用行 212 附近上下文） |


## 三、逐格判——判决第三节（攻方腿）

| 格 | 判 | 依据 |
|---|---|---|
| X2 I-5.2 恒真，改 `DeviceFreeMap` 独立 `free_slots` | 判对（副本验证，比判决更强） | 详见文末「六条变异」表第 2 条：不仅验收断言红，checker 的 I-5.2 本身也直接判 `Violated`——比判决原文「会红：把 `mark_allocated` 里的减法摘掉，checker 的 I-5.2 判 Violated」验证得更完整，两条独立路径都响 |
| X1-① defer 只增不减，处置「写明」不改代码 | **处置不合条款** | 详见「特别核 (1)」 |
| X1-② 第 50 轮 panic，改 `AllocationRecordsExceedOneNode` | 判对（副本验证） | 详见文末「六条变异」表第 3 条：把判定改成恒假后确实 panic 在 `unit.rs:160`「条目装不进一个节点：写者要先按 `index_node_entry_capacity` 判、报错，不许走到这里」，与判决「改回恒假就 panic 在 `unit.rs` 的断言上」逐字相符 |
| X4 oracle 丢实例维，改 `PublishedVersion`/`newest_persisted_root` 带实例 | 判对（副本验证） | 详见文末「六条变异」表第 5、6 条：两条会红用例各自独立验证；`crash.rs` 现查 `pub struct PublishedVersion`（549 行）`pub fn oracle_violation_for_versions`（559 行）`pub fn newest_persisted_root`（596 行，无 `_txg` 后缀——攻方腿报告写的 `newest_persisted_root_txg` 是修前的旧名，修后被判决要求的改法本身把它改名并加了实例维，这是命名演进不是错误） |
| X5-① M1/M3 只有验收断言，处置「立 C374、收窄措辞」 | 判对（门槛较低，勉强够） | `.claude/kb/checks-owed.md` C374 现查存在且文本与判决描述一致（「两条都拿写者自己的内存态输出与常量比……checker、走读、层 0 oracle 三条独立的路一个字都不说」），并给出了具体remedy（两条新空间记账不变量）与判别力自证要求；跑前条款对 X5 的要求本就低于 X1-X4（「X5 打中 ⇒ 补对照」，不要求「改代码 + 会红用例 + 再攻一轮」），把debt 显式记账 + 纠正此前「没有会红的检查」这句过宽的现状描述，是对这条较低门槛的合理满足，但 C374 本身仍是**未实现**的欠账，没有新增一条真正独立的对照检查，只是账记得更准——这一点判决没有说错，只是「补对照」三个字比实际做到的（记账 + 收窄措辞）要重 |
| X5-② 映射那一格，改 `placements_to_release_via_mapping` | 判对（逐字测到 + 副本验证） | `.claude/kb/milestone/02-second-txn.md` 第 125-126 行两条验收标准「第一个单元的两条分配记录读回……释放判定路径对它报「不在映射」」「删掉映射条目，释放判定路径报「不在映射」而不是按提示释放」与 `release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released` 测试逐字对应：前一条对应 `mapping_locations_for_key(second_mapping, first_data_key) == None`（B 覆盖写之后 A 的数据单元 key 在 B 的映射里查不到），后一条对应构造「删掉码 1 条目」的映射节点再 `try_overwrite`，断言 `Err(PublishError::ReleaseNotInMapping { unit: TransactionUnit::Data })` 且 `fresh_pool.allocator.records() == records_before`（一条记录都没改写）。副本验证见文末「六条变异」表第 4 条 |
| 顺带①（X6）分配器每次挂载从零重建 | 判对 | 现查 `grep -rn "PoolAllocator::new\|DeviceFreeMap::new" crates/singlefs-core/src/recovery.rs` 零命中，恢复路径确实没有构造分配器的地方；判决把它归入步 3 题面（第二个可写实例要重开），属于 X6 允许的「记进里程碑对应步的决策点」处置，合条款 |
| 顺带②（X7）步 2 现状「空闲 = 单元区 − 已分配」与 `transaction.rs:882` 注释不符 | 判对 | 随 X2 一起改：现查 `transaction.rs` 对应注释已改写为「第 2 项独立维护：分配器在分配那一刻减它，不由「容量 − 已分配」现算」，milestone 步 2 现状句（第 111 行）也已改写为「空闲 = 独立维护的空闲槽数……不由「单元区 − 已分配」现算」，两处口径一致 |


## 四、特别核 (1)：X1-① 与 X2 第二次抽样的处置，按跑前条款是不是「打中要改代码」

**跑前条款原文**（`_m2-code-r1-body.md` 第三节）：「反向接受条款：**X1–X4 打中 ⇒ 改代码、补一条会红的用例、再攻一轮**；X5 打中 ⇒ 补对照；X6 打中 ⇒ 记进里程碑对应步的决策点或欠账；X7 打中 ⇒ 改现状。」X1、X2 都在「X1–X4」这个范围内，条款没有为「需要实现整个未来子系统才能修」这种情况写例外，也没有把这类情况降级成 X6 的待遇。

**X1-① 的处置**（判决三·2）：「**写明**：里程碑步 2 现状加一句……指到 C283（准入失败时不先推发布就报 ENOSPC） 与 I-5.3（报出的空闲都兑现得了） 未实现那一格；不另立欠账（那一格已有）」——没有改代码，没有加任何测试（哪怕攻方腿自己在报告第七节建议的「弱形态」用例：「断言 `deferred_slots` 与 `df` 能报的空闲之间的关系今天是什么」也没有落地）。

现查 `.claude/kb/checks-owed.md` C283 那一行与 `.claude/kb/invariants.md` I-5.3 那一行：两条**在这一轮之前就已经存在**（C283 落款「2026-09-11 用户问……同日用户定案 D3（空间分配） 已定项 9 后改写」；I-5.3 落款「新增……2026-09-11 用户定案」），且两条都明写「**会红的检查仍欠**」「**未实现**」。它们确实精确覆盖了 X1-① 指出的同一个语义空白（I-5.3 的定义句「把『放回扣住量』那一步摘掉，近满盘上删一个对象再写同样大小必须由绿转红」正是可再分配谓词缺失这件事），**但它们是「已经欠着的账」，不是这一轮新增的会红检查**。跑前条款要的是「补一条会红的用例」，引用一条早已存在、早已标注「未实现」的欠账条目，不构成新增的会红检查，也不构成「改代码」。

**X2 第二次抽样的处置**（判决四·1）：「X2 第二次抽样提出『A 的根被环覆写之后并集缩、占着不缩』，记进步 5 决策点」——同样没有改代码、没有补用例，直接落成 `.claude/kb/milestone/02-second-txn.md` 步 5 决策点里的一条陈述（现查第 212 行确有对应文字）。这与 X1-① 是**同一类处置**：两者其实是同一个病根的两个侧面（可再分配谓词缺失 ⇒ defer 永不回来 ⇒ 环覆写之后账目对不上），但按跑前条款字面，它们仍然是「X2 打中」，仍然要求「改代码 + 会红用例 + 再攻一轮」。

**判据**：同一份判决里，X3 第 2 条（逐盘核）、X4（oracle 实例维）都属于「今天的测试流走不到/不可达」（X4 判决原文明写「B 这条流全是一个实例……今天不可达」），却仍然拿到了完整的「改代码 + 会红用例」待遇；X1-② 需要 50 轮覆盖写才触达，也拿到了完整待遇。唯独 X1-① 与 X2 第二次抽样，以「需要 F_生效 与环里最旧有效根这两样步 5 才有的东西」为由被降格成了 X6 式的「记决策点」。这个理由本身是有工程道理的（实现可再分配需要一整块步 5 才规划的子系统，不是局部小改），但**跑前条款没有为这条理由开例外**，判决也没有在文中明说「这里在按 X6 的待遇处理、偏离了 X1-X4 的条款」——它只是直接引用了已有的欠账条目顶上，读起来像是「条款已满足」，而实际上一条新代码、一条新测试都没有。

**结论**：这两格的技术判断（可再分配确实没实现、环覆写确实会让 I-3.1 未来红）是准确的，**但处置本身不合条款**——按「X1–X4 打中 ⇒ 改代码、补一条会红的用例、再攻一轮」的字面要求，这两格都还欠着；C283 / I-5.3 顶得上「这件事已经被诚实记录、不是静默空白」，顶不上「补一条这一轮新增的会红用例」。建议：要么把这两格的处置改成条款允许的样子（哪怕是攻方腿自己建议的弱形态用例），要么在判决里明写「这里偏离 X1-X4 的反向接受条款、按 X6 的宽松待遇处理，理由是……」，把偏离摆到台面上而不是用已有欠账条目盖过去。


## 五、六条变异逐条红在哪（副本上各跑一遍、跑完还原）

| # | 变异 | 改哪 | 红在哪（副本实测） | 与判决/报告所说是否相符 |
|---|---|---|---|---|
| 1 逐盘核 | `allocation_records_are_one_per_device` 退回旧判法（只核总数） | `recovery.rs` | `recovery::allocation_records_per_device_tests::a_placement_recorded_on_one_device_only_is_rejected_even_when_the_total_is_even` 与 `a_release_rewritten_on_one_device_only_is_rejected` 两条单测：`assertion failed: !allocation_records_are_one_per_device(&records, &BOTH_DEVICES)` | 相符（正推腿「把比较臂改回 true 它红」） |
| 2 空闲不减 | `mark_allocated` 删掉 `self.free_slots -= span;` | `allocator.rs` | `cold_start_reads_the_second_content_and_the_pool_checker_stays_green`：`I-5.2 在覆盖写之后的镜像上判红：Violated("盘 0：空闲 Some(3472883712) + 已分配 Some(376832) ≠ 单元区 Some(3472883712)")`；`release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue` 的空闲绝对值断言 `left: 3472883712, right: 3472506880` 同时红 | 比判决说的更强——两条独立路径（checker + 绝对值断言）都响，不止一条 |
| 3 装不下不判 | `records_after_this_publish > allocation_node_capacity` 恒假 | `transaction.rs` | `repeated_overwrites_report_a_full_allocation_node_instead_of_panicking` panic 在 `unit.rs:160`「条目装不进一个节点：写者要先按 index_node_entry_capacity 判、报错，不许走到这里」 | 相符 |
| 4 释放按提示 | `publish_overwrite` 释放入参改回 `&previous.placements()` | `transaction.rs` | `release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released` 红：删掉映射条目那一支返回 `Ok(...)` 而不是期望的 `Err(ReleaseNotInMapping)` | 相符（X5-② 判决所说的「按提示释放的写法会把它当成一次正常的覆盖写」） |
| 5 oracle 只比 txg | `(effective_txg, effective_instance) < (newest_txg, newest_instance)` 改成只比 `effective_txg < newest_txg` | `crash.rs` | `oracle_instance_tests::landing_on_the_lower_instance_of_the_same_txg_is_a_violation` 红：`只按 txg 比会把实例 1 的第 7 代当成最新的` | 相符 |
| 6 oracle 只按 txg 找版本 | `versions.iter().find` 去掉 `version.instance == effective_instance` | `crash.rs` | `oracle_instance_tests::the_same_txg_from_two_instances_are_two_versions` 红：`left: Some("读回的内容不对（走的是第 7 代根）"), right: None` | 相符 |

六条变异一次改一处、跑对应测试、立即还原；`grep -rn "MUTATION-" crates/` 在还原后命中 0；`cargo test --workspace --no-fail-fast` 复核回到基线全绿（无 `FAILED`）。

## 六、小结

第一、二节（本地腿、正推腿）的全部 12 格与第三节（攻方腿）的 8 格中，**技术判断没有一格判错**：没打中的确实没打中，打中的确实打中，改法都改到了打中的那一格（六条变异逐条验证会红在该红的断言/checker 上，其中 X2 的验证比判决说的更强）。

**处置层面有两格不合条款**：X1-①（defer 只增不减）与 X2 本地腿第二次抽样（环覆写之后 I-3.1 与分配器分道）都被判「打中」，但跑前条款「X1–X4 打中 ⇒ 改代码、补一条会红的用例、再攻一轮」没有兑现——两格都只是指向或新增了一条文字记录（C283/I-5.3 是既有欠账，milestone 步 5 决策点是新写的文字），没有代码改动，没有新测试（哪怕是攻方腿自己建议的弱形态）。X5-① 的处置（立 C374 + 收窄措辞）门槛较低（条款只要求「补对照」），可以算勉强合条款，但同样没有真正新增独立检查，只是把欠账记得更准。

**建议**：X1-① 与 X2 第二次抽样这两格要么在本轮补一条弱形态的会红用例（哪怕只是断言 `deferred_slots` 单调增、`I-5.3` 仍未实现这件事本身），要么在判决文字里显式承认「这里偏离 X1-X4 的条款、按 X6 的待遇处理，理由是需要步 5 的子系统」，不要让「已有欠账条目顶着」看起来像是条款已经满足。这两格不影响第一轮改代码那五处（逐盘核、空闲独立维护、装不下报错、释放经映射、oracle 带实例维）的正确性——它们各自的会红用例本轮全部验证通过，可以按跑前条款「打中要再攻一轮」的要求视为已完成本轮复核，进入下一轮前只需把这两格的处置补齐。
