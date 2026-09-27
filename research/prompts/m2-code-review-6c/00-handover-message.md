# singlefs-6c 移交的代码审阅（原样，2026-09-27 JST 01:0x 收到）

<!-- doc-lint:not-numbers C475 C512 C314 -->

主 agent 收到的跨会话消息正文，一字不改；分组报告 A–G 与 `00-summary.md` 在同目录（`SHA256SUMS`），没归档的九份源码快照见 `NOT-ARCHIVED.md`。

---

代码审阅报告移交：crates/ 核心实现、checker 与验证装置共 38 条问题，请你判去向与排期；我这边不改代码。

用户在我这个会话（singlefs-6c）里的原话：「你把检查报告发给 Milestone 2 remaining tasks 让 Milestone 2 remaining tasks 决定怎么改 怎么排期」。我只做了只读审阅，没改仓里任何文件。改不改、怎么改、排进哪一轮，由你按 main-agent.md「冒出新点先判阻塞」判。

## 口径
- 范围：singlefs-format、singlefs-core 全部、singlefs-checker 全部；singlefs-harness 只审了 lib.rs / segments.rs / crash.rs / model.rs / model_comparison.rs，其余 harness 源文件与 tests/ 没审。
- 做法：7 个只读审阅员分组逐行读（没跑 cargo），交回后我对着现行代码逐条核重点项。每条标「主核」= 我现读代码核过；「审」= 只有审阅员报告、我没现核。
- 时点：2026-09-26 14:15–14:55 UTC 的工作区，含你那边未提交的改动。审阅期间 walk.rs、crash.rs、model.rs、model_comparison.rs、fault_injection.rs、layer0_progress.rs 被改过，行号可能已漂，请按引的代码串定位。
- HEAD 对照：除标「仅工作区」的两条（18、23）外，主核条目的代码串在 HEAD（73ba4a4）里同样存在，是一直就有的，不是这轮带进来的。
- 欠账表：按关键词 grep 过 .claude/kb/checks-owed.md，只有第 15 条对上 C475（非默认环长下单元区起点取编译期常量）；其余没搜到对应条目。搜法是关键词，不保证穷尽。
- 分组详细报告（每条带原行引用、失败场景、注释与代码不一致表）：/tmp/claude-1000/-home-fy5090-code-singlefs/e0d0c045-b2b9-463b-958e-930037b4bce3/scratchpad/review/ 下 A-report.md 到 G-report.md。A format 与编解码；B 分配器、准入、块设备；C 事务层；D 挂载与读路径；E 恢复与树；F checker（含 45 条不变量的落点表）；G 验证装置（含崩溃点枚举的「枚举了什么 / 省略了什么」两张表）。读不到那个目录也不要紧，下面是全部结论。

没问题的部分：clippy --workspace --all-targets -D warnings 退出码 0；核心 crate 无 unwrap、无会丢值的 as；发布路径「单元 → 屏障 → 记录 → 屏障 → 根槽 FUA → 系统配置轮换」次序、冻结重发、失败换回分配器都核过没问题；编解码两侧字段逐项对称，CRC 正确；读路径每一跳验校验和与出生身份。

## 一、验证装置与 checker 的盲区（决定层 0 与 45 条全绿能证明多少）
1. [主核] 录制器合并相邻屏障、切段不分设备。harness lib.rs 的 push：`if operation.kind == RecordedOperationKind::Barrier && previous_is_barrier { return; }`；crash.rs 切段 `RecordedOperationKind::Barrier =>` 与 segments.rs:104 只看种类。池屏障是 transaction.rs:329 `for (_, device) in self.devices.iter_mut() { device.barrier()?; }` 逐盘发。把这个循环改成只发盘 0，录制流逐字相同，层 0 与崩溃注入照绿；「一块盘副本没落而根已持久」这类状态从不生成。mutations.tsv 里没有盯逐盘屏障的变异。
2. [主核] 崩溃状态上只跑只读 recover。crash.rs `let consulted = recover(&image, JournalPolicy::Consult);` 与 Ignore 两遍；crash_injection.rs 里 mount 零命中。D13（验证路线） 已定项 7 定案「崩溃后镜像是两份」、比对对象是实现恢复后的镜像，代码里没有这一份；取号、写行、暖机在任何枚举出的崩溃状态上都没跑过，二次崩溃不在枚举域。
3. [主核] FUA 关段按池算。crash.rs `if retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess { segments.push(...) }`，关段后同段别的盘上的普通写被当成已持久。今天发布路径里 FUA 独占一段，没触发，潜伏。
4. [审] 撕裂并进「没持久」对原地覆写不成立（系统配置槽、journal 环回绕）：原地覆写撕裂时旧内容也没了，层 0 从不生成「这一槽新旧都读不出」。与 D13（验证路线） 已定项 4 射程「撕裂态与没持久的差别只有 CRC32C 碰撞概率」那句冲突：要么补三态枚举，要么把它写成承认的省略。
5. [主核] checker 判不出交叉链接。walk.rs 的 visited_units 整个 Walk 共用，同一根里第二次引用同一单元直接返回（505 附近、674、989、1112、1262），五元组、诞生身份、树 ID 全跳；note_reference 按 `(location.device, location.slot, span)` 去重。两个 inode 的 extent 指向同一数据单元时 I-5.1、I-3.1、I-9.10 全绿。HEAD 同形。
6. [主核] checker 对构造的坏镜像会 panic：walk.rs:718-724 `let key_width = view.key_width;` 后切 `entry[key_width..key_width + node_pointer_bytes()]`，而 checker lib.rs 的 index_node_view 只拒 `entry_width < key_width`；walk.rs:4058 `records.get(&(counter - 1))` 在计数器 0 上下溢（同文件 4399 用了 checked_sub）。checker panic 等于那个状态上 45 条都没判。
7. [主核] I-7.4 只判回退候选集：walk.rs:4787 `let walked = *index == newest_index || (!abandoned && !below_floor);`，被抛弃的根整批不判；invariants.md I-7.4 原文写它们引用的块「在离开根环之前同样不许重新分配、不许抹头」（C314 影子账）。
8. [主核] 中央映射条目只核校验和与升序：walk.rs walk_central_mapping_entries 不把映射 key 与被指单元的类标签、出生身份比，也不进 references；类标签 0、4…255 也按 32 KiB 读。
9. [审] 查窄的不变量：I-2.5 journal 点名项那一半解出来就丢（checker 里 `.named` 零命中）；I-9.4 三句只判第一句（walk.rs:1015）；I-9.2 把合法的类型段 0 判违例（今天实现不让 inode 树长高，不可达）；I-3.9 / I-5.4 不读根记录直接持有的那棵分配记录树（C512）；I-7.7 一个槽读不出就 ① ② 一起报不适用。F-report 末尾落点表：25 条查全、20 条查窄。
10. [主核] 稀疏盘不判越界：crash.rs `impl BlockDevice for SparseBlockDevice` 的 write_at 直接 `self.image.write(offset, bytes); Ok(())`，SparseDevice::write 只断言对齐；reader 越界返回 Some(全 0)，与 PoolReader 契约「越界返回 None」不符。
11. [主核] 带算术的 const fn 被实现、checker、模型三方共用：singlefs-format lib.rs `pub const fn index_node_header_bytes`（86 + 2k + 29），core unit.rs 4 处、checker lib.rs 2 处、model.rs 2 处。与 D13（验证路线） 已定项 5「发射物里不许有分支与算术」冲突。
12. [审] 对拍单向、不比内容：model.rs judge_allocation_generations 只遍历实现交回的角色，漏交不报；ObservedRoot 没有文件内容与实例表字段，每一版的内容只在冷启动那一步比。
13. [主核，实跑] 门禁 59 号认不出 should_panic 测试。mutations.tsv 第 462 行点名 root_ring_occupancy_that_missed_the_root_panics_instead_of_reclaiming_by_the_guessed_ring（allocator.rs 里带 #[should_panic]）。我跑了 `cargo test -p singlefs-core --lib -- root_ring_occupancy_that`，输出行是 `test allocator::tests::root_ring_occupancy_that_missed_the_root_panics_instead_of_reclaiming_by_the_guessed_ring - should panic ... ok`；59 号 397 行的 red_pattern / ran_pattern 都要求名字后面紧跟 ` ... `，多了 ` - should panic` 就匹配不上，这一行会被判「点名的测试没跑到」。59 号本身我没跑。
14. [审] 并片单测没钉 failed_states / verification_failed_states 两项相加（crash.rs 并片用例里两项都是 0）；恢复用的 journal 提示不按记录槽对齐过滤，与真盘全环扫描不等价。

## 二、核心实现：静默出错或与条款不符
15. [主核，已登记 C475] journal 环长是 mkfs 参数，单元区起点是常量 784 MiB。make_filesystem.rs check_geometry 按 `(JOURNAL_RING_START_SLOT + ring_bytes / SLOT_BYTES) * SLOT_BYTES` 判，实例表、树表、分配器用 UNIT_AREA_START_SLOT = 50176。环长 > 768 MiB 时环盖住实例表单元，mkfs 不拒；环长 < 768 MiB 时判据放过，清零之后写到 784 MiB 才报 OutOfRange。journal.rs record_offset 在环长 < 4096 时 `% 0` panic。
16. [主核] 固定结构几何没有互不重叠检查：槽距 = 4096 向上取整到 io_min，无上界；check_geometry 只判 `root_ring_end > smallest`。区域 0 与区域 2 同在盘 0、起点差 6 MiB，S × 槽距 > 6 MiB（S=8 时槽距 > 768 KiB）就重叠；槽距 ≥ 1 MiB 时系统配置槽 1 落在根环区域 0 槽 0。pbs < 457 在 mkfs 中途 assert panic（这半句审）。
17. [主核] 可写挂载不核调用方参数与盘上系统配置：读与判定用 choose_system_configuration 读出的那份，写入口 `PoolWriter::new(parameters, devices.as_mut_slice())` 用调用方的；transaction.rs:376-382 取号写系统配置时 `sizes: self.parameters.geometry`、`region_devices: self.parameters.region_devices`，盘上「不可变」段会被改写。mount.rs 里没有一处比对两者。
18. [审，仅工作区] 同一设备身份交两次不拒：distinct_device_identities_handed_in 只数不同身份，取号按下标逐盘写（transaction.rs:660）；`[(d0,A),(d1,B),(d0,C)]` 放行，C 收到本池系统配置写。这个函数 HEAD 里没有，是未提交改动里的新代码。
19. [主核] 取号与抬 F 的系统配置写之前没有屏障：transaction.rs write_acquired_instance 与 write_the_raised_floor_into_every_system_configuration 第一个写之前无屏障，上一次发布的 RotateSystemConfigurationSlots 之后也没有尾随屏障；取号回卷前后都没有屏障。两槽轮换靠「覆写旧槽时新槽已持久」。C 组审阅员引 layout/01-first-txn.md:412 自己登记了「抬 F 先写系统配置、与 txg 14 那次轮换合成 4 写一段」。补屏障会动段序列登记表（门禁 52 号）。
20. [主核] 准入 ckpt_cost 按树高计、实写更多：admission.rs checkpoint_cost_of_the_version_to_build_on 对分配记录树取高（两盘 4 GiB 是 3），而 tests/second_transaction_supplement_one_write_accounting.rs 钉 `ALLOCATION_RECORD_TREE_NODES_REWRITTEN: u64 = 5`（按盘分路，每盘各改一条路径）。代码照的是 D28（挂载期承诺量） 已定项 4 字面「Σ 每棵记录树当前的高」，所以是条款与实写口径分叉，要定改条款还是改实现。「保留池每盘少扣约 2 块」是审阅员推的，没量。
21. [主核] 只读挂载打不开新建的空 inode：mounted_read.rs:715 `data_unit_count_of_a_sequential_write(inode_record.size)` 对长度 0 返回 1（write_request_split.rs `.max(1)`），而没有 extent 条目时记录数是 0；transaction.rs publish_new_inodes 建的 inode 都是 `size: 0`、不写 extent。全仓 open_file 只开 FIRST_INODE_NUMBER，没用例碰到。
22. [主核] 重建影子账时第二次读实例表失败就无声关掉隔离：mount.rs:852 `let newest_table = choose_root(...).and_then(|newest| instance_table_of_root(devices, &newest));`，None 时 is_abandoned 恒 false，一个槽不隔离，abandoned_roots_unreadable 仍 0；前面 rebuild_previous_version 已经读出过这张表。
23. [主核，仅工作区] 抬 F 途中的块设备错被归成「推满仍不够」：FloorRaiseStop::FloorRaiseFailed(Box<MountError>) 进 MountSpaceAdmission::StillShortAfterTheFloorRaises，mount_writable 仍返回 Ok；mounted_session.rs 把它包成 UserChangeRefused::NoSpaceAfterRaisingTheFloor。HEAD 里没有这段，是未提交的抬 F 准入流程带进来的。
24. [主核] 可写挂载不跑冷走读：core 里 walk_to_file 零调用（mount.rs、mounted_read.rs 只在注释里提到），可写路径靠 rebuild_version，它以 CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds 读记账树与映射树、不判 key 次序。「I-9.2、I-7.8、树头出生身份在这条路上没人判」是审阅员的。
25. [主核] journal 反向链解出来没人用：journal.rs:288 `let back_chain = reader.get_u32();`，recovery.rs 里 back_chain 零命中；与 I-8.6「不等的记录不进重放前缀」及 checker 的前缀口径不一致。要么补检查，要么改条款文字。
26. [主核] 一次发布的 journal 记录数没有上限：transaction.rs 里没有一处读在飞上限或环槽数；journal.rs:84 的 expect 消息说「按 extent 叶容量截过」，那道截断已不存在。超过环槽数时会盖掉自己的记录与所选根的锚点记录（后果是推的）。
27. [主核] 树表里同一种树出现两条时读者取法不一：rebuild 取首条（recovery.rs:1317 附近 find）、allocation_records_under_root 与 walk_to_file 取末条（1056、2513 附近赋值覆盖）、user_visible_tree_root_pointers 报错（1768）、mounted_read.rs:409 取首条非空。
28. [主核] compat_ro 读者整段跳过：crates 里 compat_ro 零命中，system_configuration.rs parse_slot 只切 incompat。今天一位都没分，暂不触发。
29. [主核] 指针与单元头的加密、压缩、29 字节预留位读者不判恒 0：pointer.rs `reader.skip(16 + 12 + 1 + 1 + 2 + 2);`，unit.rs 跳过预留位；与 I-2.4「非 0 判损坏」不符。系统配置的格式版本、加密类型（偏移 219）、结构常量读者也不看（这半句审）。
30. [主核] 分配路径代价随盘容量线性：allocator.rs lowest_user_data_slot 从单元区起点逐偶数槽扫、每个候选对聚簇段全表 any；make_room_for_record_on_device 每次扫全部记录；每次发布 clone 分配器（1 TiB 约 6400 万槽、三张 Vec<bool> 约 190 MB 是审阅员推的，没量）。与 fs-design.md「运行时决策路径…代价不许随盘容量增长」字面冲突，没登记欠账。
31. [审] 其余：B 组第 4 条记账行「全空聚簇段数」与 D3 已定项 10 ① 定义两处不一致；C 组第 3 条两槽都读不出时世代号从 1 起；C 组第 4 条发布入口不核接在最新那一版、同一实例；D 组第 7 条新实例第一次发布的 txg 没与所选根取 max；E 组第 8 条池成员按交进来的盘算、不按系统配置；B 组第 7 条 open_existing_image_file 对已有镜像 set_len。

## 三、构造的坏盘会 panic（都要 CRC 自洽、内容错的镜像才触发；核心层读坏盘应报错）
32. [主核] 被抛弃根的指针槽号只判「两条位置条目同槽」就进分配器：mount.rs:789 `slot_shared_by_both_location_entries(&pointer.locations).ok()?`，之后 allocator.rs isolate 的 index() `checked_sub(...).expect` 与 `assert!(end <= self.allocated.len())`；mount.rs format_time_allocator 的 mark_format_time_units 同理（审）。
33. [主核] 重建跳过 key 次序、发布规划 assert：recovery.rs 497、1509 附近用 OnlyWhatTheShapeNeeds；code_two_tree.rs 叶的严格递增只在 judges_every_header 时判；transaction.rs:6468 `assert!(planned_keys.len() == leaf_entry_bytes_by_key.len() && ...)`。有重复 key 的记账叶让挂载后第一次发布 panic。
34. [主核] 释放 mkfs 树表只看第一块盘：transaction.rs format_time_tree_table_to_release 的 still_allocated 用 `allocator.devices.first()`，之后 allocator.rs release 对每块盘 expect、assert!(!record.is_released)、assert_eq!(span)。映射那一路有 placement_to_release_after_checking_every_device，这一路没走。
35. [主核] 盘 < 784 MiB 时 allocator.rs unit_area_slots_of_device 里 `absolute_slot_count_of_device(device_bytes) - UNIT_AREA_START_SLOT` 下溢，recovery.rs:1014 调它之前没有盘容量守卫。
36. txg / 实例代号 + 1 溢出：[主核] recovery.rs:1971 `CheckpointTxg(root.checkpoint_txg.0 + 1)` 在循环前无条件算，txg = u64::MAX 的根让只读挂载 panic；[审] mount.rs:616、transaction.rs 597-601 与 4298/4356/4409/4468 同类，而同文件 issued_from_watermark 用了 checked_add。
37. [主核] 槽 0 自证不过时按 4096 找槽 1：recovery.rs 603-625 用 FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES，槽距 > 4096 的池上两盘槽 0 同坏就整池报无有效配置。
38. [审] 盘上 journal_ring_bytes 与 physical_block_size 不设界：scan_journal 一次性 collect 环长 ÷ 4096 个偏移，读根槽按 pbs 分配缓冲。条目宽 0 而条目数非 0 的节点照收：core unit.rs 与 checker lib.rs 同写法 `chunks(width.max(1)).take(count)`，两份独立解析器在这一格一起漏。

## 四、低级
死代码：records.rs mapping_key_sort_key（它那条测试测的不是映射树真用的 CENTRAL_MAPPING 序）、transaction.rs PublishShape / ResolvedPublish::shape / ExtentTreePlan::roles、checker DecisionWidth、format 的 FIRST_TRANSACTION_ACCOUNTING_ROWS 与 FIRST_TRANSACTION_TREE_TABLE_ENTRIES、mounted_read 三个取值方法、extent_tree child_holding_data_unit。恒真测试：code_two_tree.rs `assert_eq!(u8::MAX.checked_add(1), None)`；transaction.rs every_transaction_unit_names_its_class_tree_and_placement_rule 里 tag().len()==2，且没有一条 unit_class 断言。make_filesystem.rs:369 `let _ = JOURNAL_RECORD_BYTES;` 压未使用 import 警告。注释与代码不一致七份报告共约 60 处，各在报告末节表里。

## 五、涉及已定条款、不只是改代码的几条（照 three-way-inference 可能要走三方或交用户）
- 第 20 条：ckpt_cost 条款字面 vs 实写节点数（D28 已定项 4）。
- 第 25 条：反向链 vs I-8.6 / D23 已定项 7 的恢复算法。
- 第 2 条：层 0 不跑写恢复 vs D13 已定项 7。
- 第 4 条：撕裂射程 vs D13 已定项 4。
- 第 11 条：带算术的 const fn vs D13 已定项 5。
- 第 7 条：I-7.4 被抛弃根那一半 checker 没判（C314）。
- 第 30 条：分配路径线性 vs fs-design「记账是事务的副产品」第一格。

我这边没有在跑的东西，也不会再动这些文件。有要核的点直接给 singlefs-6c 发消息。
