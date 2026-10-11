# 删除线第 60、61 批 代码轮第一轮：云端正推腿（Sonnet）报告（2026-10-11）

立场：核代码与测试做的是不是条款、判决与调查报告点名的那件事。引码写冻结副本 `~/.local/share/singlefs/work/m3-batch60-61-code-r1/repo/` 里的路径与行号，引 kb 写仓里的文件与行号，行号都是这一次 `grep -nF` 现取的。

## 0　做法与边界

- 冻结副本的被判文件 sha256 与 `research/prompts/m3-batch60-61-code-r1-snapshot/crates-sha256.txt` 逐个对过：`crash_amplification.rs`、`verdict_store.rs`、`crash_amplification_pipeline_records_checks_and_reuses.rs`、`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`mutations.tsv` 五个都相同（`sha256sum` 输出与快照文件逐行一致）。
- 我自己的量全在草稿目录 `~/.local/share/singlefs/work/m3-batch60-61-code-r1-sonnet/`：`repo/`（冻结副本的拷贝）与 `target/`（我的编译目录）交回前已删，`tools/apply.py` 打变异（行号取冻结副本 `crates/mutations.tsv`）、`tools/runm.sh` 经 `run-with-memory-cap.sh 8G` 加 `capped.sh 15` 跑 harness 档测试。只跑 `-p singlefs-harness --test <目标>`，没跑任何 checker 档测试、没起 GPU、没跑门禁阶段。每次变异后用 `python3 tools/apply.py restore` 还原，收尾 `diff -rq repo/crates ../m3-batch60-61-code-r1/repo/crates` 无输出（副本与冻结副本相同）。
- 能复跑的：P2、P3 里 harness 档的测试与变异。复核不了的：P1 的整条（要卡）、P4、P5 的测试与变异行（checker 档，`--lib` 与 `--test` 都在 `singlefs-checker-tier`），变异行 84、2728、3959 的目标也在 checker 档。这些格的结论是读码加算术，写明了。
- kb 条款面先报一件（附带发现 A）：背景材料列的条款 I-9.37 不在 `.claude/kb/invariants.md` 里。命令与输出：

```
$ grep -c 'I-9.37' .claude/kb/invariants.md
0
```

  I-9.22 在 `.claude/kb/invariants.md:310`：「每个打包记录类型 1 容器记录数 ≤ 旋钮值」，这一行只写「≤ 旋钮值」，没有「最右那片之外每片记录数 == 旋钮值」那半句。P1 的新断言依赖的恰是那半句，它的出处是代码 `crates/singlefs-checker/src/walk.rs` 与第 57 批容器判器报告（`~/.local/share/singlefs/work/m3-batch57/container-judges/report.md`），不在 kb 里。推翻它的现象：`grep -n '最右那片之外' .claude/kb/invariants.md` 有命中。

## P1　GPU 远左短片与精确红集（甲 N1）

判定：一致（读码；卡上的那一半复核不了）。

**1.1 新三条是不是只因旋钮支红。** 造法：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:14128`：「records_per_container_of_each_record: &[2, 2, 1, 1, 2, 2, 2, 1],」与 `…:14129`：「record_counts_in_leaf_order: &[2, 1, 2, 2, 1],」；造完转根环后 `…:14253`：「tree.record_counts_of_each_container(&pool),」按叶序核一遍形状，这条在没有 GPU 的机器上也跑（它排在取卡之前）。harness 那一侧同形：`crates/singlefs-harness/tests/checker_known_bad_images.rs:17091`：「const A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST: ContainersUnderChangingKnobs =」，数列 `[2, 2, 1, 1, 2, 2, 2, 1]` 与 `[2, 1, 2, 2, 1]` 两边相同。旋钮只给被写的那一棵：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13047` 起三个 `knobs_of_the_*_tree`，另两棵 `None`。CPU 新入口装了旋钮的那一棵，三类违例都记到旋钮那一条：`crates/singlefs-checker/src/walk.rs:12086`：「let invariant = capacity_invariant_of(capacity_knob);」（墓碑，I-9.22）与 `crates/singlefs-checker/src/walk.rs:11992`：「let invariant = lineage_or_retirement_capacity_invariant_of(knob_records_per_container);」（谱系、退役，I-9.37）。没装旋钮的那两棵走产品支，`crates/singlefs-checker/src/walk.rs:12089`：「Some(_) => 1,」那一段说明装了旋钮时一片就判、没装要两片以上才判，这几条流上另两棵至多一片。所以红集里只会出旋钮那一条，谱系与退役两棵共用 I-9.37 的编号，但没装旋钮的那棵若红会出 I-9.36，红集断言能把它挡出来。

**1.2 精确红集会不会在正确判法下因造法顺带的别的违例而红。** 断言在 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:14289`：「std::collections::BTreeSet::from([knob_capacity_invariant]),」。读码推不出别的违例：流是产品写路径合法写出来的，状态是每个段的真子集（段长 ≤ 8 的展开，`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:182`：「const DEFAULT_EXPAND_UP_TO: usize = 8;」）。直接证据只有第 59 批攻方替身的数，我读了原文没复跑：`research/prompts/m3-batch59-code-r1-opus-model/expected/base-m1-l2.txt` 第 6–8 行，三条流 `states_with_another_violation=0`（谱系 513 态、退役 693 态、墓碑 609 态）。复核不了的原因：CPU 红集由 `compare_on_every_state_given_the_container_capacity_knobs` 在取到 GPU 卡之后才算（`…:14277`：「let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {」在断言之前 `return`），没有 GPU 的机器上这条精确红集断言不跑，只有主 agent 在卡上那一趟才判得了。
推翻 1.2 的现象：卡上跑新三条，`cpu_checker_red_invariants` 多出旋钮那一条之外的编号（调查报告「推翻条件」第一条同）。

**1.3 k = 2 那三行变异红在哪一条断言上。** 锚点：`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3374`：「if (d_container_record_count(container_index) != knob_records) {」，全仓只命中这一处（`grep -cF` 输出 1，`grep -nF` 在 `src/*.wgsl` 全部命中 1 行）。它在循环 `…wgsl:3373`：「for (var container_index = 0u; container_index + 1u < d_container_count; container_index = container_index + 1u) {」里，循环只走到倒数第二片。变异在这一句前加 `container_index + 3u >= d_container_count &&`，等于只判最右之前两片。变异只动 GPU 内核，CPU 红集不变，所以 `…:14289` 那条红集断言不会被它打红；红在 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:14294`：「comparison.mismatches.is_empty(),」。理由：[2, 1, 2, 2, 1] 共 5 片，容器 2 是下标 1，`1 + 3 >= 5` 不成立，被跳过；CPU 在根环转过之后的状态上红它，GPU 不红，逐位不一致。三行各点名一条新测试全名（`crates/mutations.tsv:3954`–`3956` 第六列），过滤串不是 L1 名字的子串（脚本核过：三行第六列各在测试文件里恰好定义一次 `fn`，过滤串各恰好命中一个 `fn`）。复核不了：打变异后在卡上是不是真红，要卡。

**1.4 L1 三条收成造法结构之后，第 59 批那三行还点不点得中、红不红。** 第 59 批三行（`crates/mutations.tsv:3849`–`3851`）的锚点是同一句（`crash_judge_checker_scan.wgsl:3374` 命中 1 次），替换成 `container_index + 2u == d_container_count && …`（只判倒数第二片）。L1 的测试函数名一个字没动（上面脚本核：第六列各命中 1 个 `fn`），造法常量 `A_SHORT_CONTAINER_BEFORE_A_FULL_ONE` 的数列与改前相同，改前常量见 `research/prompts/_m3-batch60-61-code-r1-diff.md` 里被删的两行 `[2, 2, 1, 1, 2, 1]` 与 `[2, 1, 2, 1]`。[2, 1, 2, 1] 共 4 片，容器 2 下标 1，倒数第二片下标 2，变异只判下标 2，下标 1 被跳过，GPU 不红、CPU 红，仍红在 `mismatches` 那条。红集断言是 CPU 一侧，变异碰不到它。推翻 1.4 的现象：卡上 L1 三条打第 59 批变异不红。

## P2　水位统计量 7 与分配记录判序（乙 N2、N3）

判定：N2 一致；N3 一致（两条都复跑了变异，量数在下面）。

**2.1 补进来的 (7, 0, 0xFFFF_FFFF, txg) 会不会被读者在别的理由上先拒。** 条款面：`.claude/kb/decisions/05-快照-空间记账机制.md:124`：「| 7 | 扩展点配额已用量 |」这一行写着「不带」设备维、「不带」树维；`.claude/kb/decisions/05-快照-空间记账机制.md:204`：「不带设备维的设备段写 0xFFFF_FFFF、不带树维的树 ID 段写 0」；同一行「没有对象的统计量不写行」。测试里的行：`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:605`：「statistic: STATISTIC_EXTENSION_POINT_QUOTA_USED,」、`…:607`：「device: DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION),」，常量 `…:541`：「const STATISTIC_EXTENSION_POINT_QUOTA_USED: u16 = 7;」，设备段常量值 `crates/singlefs-core/src/records.rs:31`：「pub const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;」，树 ID 写 `TreeIdentifier(0)`、seq 取被删那一行的 seq（非 0）、代段 = `overwrite_txg`。读者今天依次判四关（`crates/singlefs-core/src/recovery.rs`）：条目宽 `crates/singlefs-core/src/recovery.rs:4773`（`every_accounting_entry_is_as_wide_as_its_field_table`，34 字节，补进来的行宽对）；逐条判 `crates/singlefs-core/src/recovery.rs:4876`：「if entry.generation > root.checkpoint_txg || entry.sequence == 0 {」（代 = 根 txg、seq 非 0，过）；数条目 `crates/singlefs-core/src/recovery.rs:4786`（`accounting_entry_count_is_three_plus_six_per_device`：不算台阶行后 15 行 = 3 + 6 × 2 盘，过）；然后才是水位行 `crates/singlefs-core/src/recovery.rs:2937`：「if !accounting_entries.iter().any(|entry| {」→ `crates/singlefs-core/src/recovery.rs:2940`：「return Err(RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree.into());」。测试用 `assert_eq!` 钉具体失败成员（`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:648`：「RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree」，可写挂载那一侧 `…:671` 同），所以在别的理由上先拒会红，不会悄悄绿。
复跑（冻结副本拷贝上，harness 档）：基线 `test result: ok. 1 passed`（`~/.local/share/singlefs/work/m3-batch60-61-code-r1-sonnet/base-n2.log`）。两个读者改法（自造，各改 `crates/singlefs-core/src/recovery.rs:4876` 一行）：

| 读者改法 | 结果 | 说明 |
|---|---|---|
| 逐条判 I-3.13：非台阶行代段不等于根 txg 就拒 | `exit=0`，`1 passed`（`mutNa.log`） | 补进来的行代段 = 根 txg，照报水位行缺：N2 要的「第 57 批两种读者改法与逐条判 I-3.13 都照报」成立 |
| 读者拒统计量 7（拿它当没登记的标签） | `exit=101`，`left: Some(Walk(InvariantViolated { … "记账条目的代晚于根或 seq 为 0" }))`、`right: Some(Walk(InodeNumberWatermarkRowMissingFromTheAccountingTree))`（`mutNc.log`） | 读者将来判「没有对象的统计量不写行」，这条用例红在别的成员上，看得见；测试注释自己写了「要换造法」 |

推翻 2.1 的现象：上表第一行（I-3.13 逐条判）下这条用例变红。

**2.2 「台阶行之外每一行代段等于这一版 txg」这句前提断言对不对。** 断言在 `crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:628`：「.all(|row| row.generation == overwrite_txg),」，消息 `…:629`：「台阶行之外每一行的代段都是这一版的 txg（I-3.13 照过）」，过滤条件把「统计量 12 且代段早于 txg」的行排出。条款：`.claude/kb/invariants.md:155`：「inode 号水位那一统计量的台阶行不判」，`.claude/kb/decisions/05-快照-空间记账机制.md:53`：「I-3.13（记账行代段等于这一版的 txg） 对台阶行同样不适用」。覆盖写那一版 16 行 = 15 行新池写法加一行台阶 (12, 3)（用例自己 `assert_eq!` 过 `16` 与 `vec![CheckpointTxg(3), CheckpointTxg(4)]`），删自己那行 (12, 4)、补 (7, 0, 0xFFFF_FFFF, 4) 仍 16 行，只剩台阶一行水位；其余 15 行代段 = 4。基线绿即证这句前提成立。

**2.3 N3 那张盘是不是只由「分配记录判在条目宽之前」分得出。** 盘：`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:2043`：`fn pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_root_is_narrowed_to_its_keys(`，先核前提 `…:2052`：「前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目窄于字段表」，再坏分配记录。两条用例 `…:2176`（冷走读）、`…:2195`（可写挂载），各钉报分配记录的代晚于根、不报条目窄于字段表（`…:2187`、`…:2202`）。读者里这两个次序：`crates/singlefs-core/src/recovery.rs:5395`：「allocation_record_generations_and_spans_are_judged(allocation_records, root)?;」（冷走读）、`crates/singlefs-core/src/recovery.rs:2894`：「allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;」（重建），都在条目宽（`crates/singlefs-core/src/recovery.rs:4773`）之前。
复跑：基线 8 条同前缀用例全绿（`base-n3.log`：`8 passed; 0 failed`）。打变异表这两行（`crates/mutations.tsv:3957`、`3958`，锚点各命中 1 次，我用 `tools/apply.py row N` 套同一对原文与替换文）：

| 变异 | 8 条里红的 | 其余 |
|---|---|---|
| 3957（冷走读挪到条目宽之后） | 只有 `…_before_the_accounting_entry_width_by_the_recovery` | 7 passed（`mut3957.log`） |
| 3958（重建挪到条目宽之后） | 只有 `…_before_the_accounting_entry_width_by_the_writable_mount` | 7 passed（`mut3958.log`） |

所以两行各只红自己那一条，理由成立；老的两张盘（条目宽都对）在这两个变异下仍绿，说明分得开这一形的只有新盘。注意：变异名写「挪到条目宽之后、逐条判之前」，但新用例对任何落在条目宽之后的位置都红（挪到逐条判之后、数条目之后也是），它钉的是「在条目宽之前」，不钉具体落在哪一关之前。推翻 2.3 的现象：把分配记录判挪到条目宽之后任一位置，新两条仍绿。

## P3　轻型核心行的修补（丙 N4）

判定：两条新取样点在测变异行点名的那件事，一致；清零再写回那一手部分是造的（见 3.2）；8 行替换里 harness 档 4 行与改名那行之外的 3 行属 checker 档，复核不了。

**3.1 取号回卷之后的屏障。** 用例 `crates/singlefs-harness/tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs:197`：`fn acquisition_after_a_rolled_back_acquisition_overwrites_the_older_slot_only_behind_a_barrier() {`；盘 1 的取号写报错、盘 0 回卷（`failure.rollback` 断言 `RolledBack`），撤掉故障后同一个写入口再取号（`…:238`：「acquire_instance(&mut writer).expect("故障撤掉之后，同一个写入口再取号做成");」），最后 `…:246`：「"mkfs 之后没有一段里同一块盘写两次系统配置槽"」。回卷写后面没有屏障，再取号的第一个取号写与回卷写之间只剩取号自己那道屏障，所以改坏取号写之前的屏障（`crates/mutations.tsv:857`「实审A1 第19条：取号写之前不过屏障」）就同段写两次。
复跑：基线 4 条全绿（`base-n4a.log`：`4 passed`）。打 `crates/mutations.tsv:857`：`exit=101`，红的只有 `acquisition_after_a_rolled_back_acquisition_…`，另三条（含 `rolling_back_a_failed_acquisition_…`、`acquisition_after_a_publish_of_the_previous_process_…`）`ok`（`mut857.log`：`3 passed; 1 failed`）。推翻现象：打 857 后这一条绿，或另三条里有一条红。

**3.2 影子账一圈窗口在抬 F 那一路的判别力，与「清零再写回」是不是硬造。** 用例 `crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs:599`。机理：窗口过滤在可写挂载那一遍不敏感，因为窗口外那几槽还在 defer 里、分配位置着，那一路只隔离分配位没置着的槽（`crates/singlefs-core/src/mount.rs:1650`：「allocator.isolate_abandoned(device, slot, span_slots);」与它前面 `reclaim_isolates_what_this_root_references` 的两个分支，`mount.rs:1601` 先取根环表里记着的落点）；根环表里没记着这条被抛弃根时，才走整段隔离。新用例把 24 的根槽在第二次可写挂载那一刻读成全零（`…:629`：「&vec![0u8; root_slot_bytes],」），交回后原样写回（`…:653`：「&root_slot_of_the_abandoned_root,」），再在同进程里抬 F（`…:677`：「raise_rollback_floor(」），断言窗口外候选与它同引用的槽一个都不隔离（`…:701`：「Vec::<(DeviceIdentity, u64)>::new(),」）。
复跑：基线 3 条全绿（`base-n4b.log`：`3 passed`）。打 `crates/mutations.tsv:2755`（影子账候选过滤改回一圈窗口）：`exit=101`，红的只有新用例，老的 `a_remount_isolates_no_slot_…` 与另一条 `ok`（`mut2755.log`：`2 passed; 1 failed`）——测试注释里「那一条对这个变异不敏感、由新用例抓」的说法被量证实。
是不是硬造：清零再写回是「根槽在挂载那一刻一时读不出」的建模，同一手在这份文件里造被抛弃根时已经用了（`…:355-362`，`pool_with_the_newest_root_of_instance_one_abandoned` 里把最新根槽清零，挂载交回后原样写回），读者里对应的是账读不出的处置（`mount.rs` 里 C393 取 (b) 那段）。产品能走到「表里没记着这条被抛弃根、盘上却读得出」的只有瞬时读不出这一形，所以用例造的是这一形，不是产品走不到的状态，但回写那一手是测试替瞬时故障撤销，不是产品里的一步。推翻现象：能指出产品里「表里没记着、盘上读得出」的状态不会出现。

**3.3 8 行整行替换与改名那一行：在原条款下是不是真会改行为、轻型里跑不跑得到。** 我只能复跑落在 harness 档的行（checker 档不跑）。每行套 `crates/mutations.tsv` 里那一行的原文与替换文（`tools/apply.py row N`，锚点各命中 1 次），跑第五列点名的目标与过滤串：

| 变异行（冻结副本 `crates/mutations.tsv` 行号） | 目标 | 结果 |
|---|---|---|
| 65 增补 1：发布路径漏计数据单元的写 | harness `write_accounting_per_structure` | `FAILED`，`0 passed; 1 failed`（`mutrow65.log`） |
| 396 并行线一：C490 offset 段写成单元序号 | harness `sequential_write_split_into_one_unit_transactions` | `FAILED`（`mutrow396.log`） |
| 657 实二七：有盘不带所选那一版也照样取号 | harness `fsync_drop_and_devices_without_the_selected_version` | `FAILED`（`mutrow657.log`） |
| 2232 C673 ⑤：头 ID 一律写字面 65537 | harness `pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement` | `FAILED`，`0 passed; 1 failed; 8 filtered out`（`mutrow2232c.log`；我头一次把目标取成了名单里最后一个 `checker_known_bad_images`，过滤后 0 条跑，那一次作废，改点名第一个目标重跑） |
| 84 步 6 失配即中止 | checker 档 `crash_enumeration_fixed_script_stream` | 复核不了（checker 档） |
| 2728 E171R1 装置 U12 | checker 档二进制 `e171_three_oracle_collusion_after_the_oracle_fixes` | 复核不了（checker 档） |
| 3959 提交标记提前到第一条记录（改名那一行） | checker 档 `crash_enumeration_multi_record_publish_stream` | 复核不了；读码：`crates/singlefs-core/src/transaction.rs:9155`：「let place_in_publish = if transaction_of_the_next_record.is_none() {」换成 `if true`，一条记录的发布里这个条件本来就真，≥ 2 条记录的发布每条都被标成最后一条，字节变了，是真改行为 |

（头一次把行 84 的目标套在 `-p singlefs-harness` 下，cargo 报没有这个目标就退了，没编也没跑任何 checker 档的东西。）
轻型登记表：`.claude/gate.d/crates-mutation-light-rows.tsv` 共 172 行，我用脚本核：每行第一列都在冻结副本 `crates/mutations.tsv` 里，每行第二列等于那一行第五列 `--` 之前那段，不符 0 行；第 54 行：「并行线一：层 0：一次多条记录的发布每条记录都带末条标志」就是改名那一行。该文件 sha256 `d8c887bc…1035` 与快照一致。哪几条在轻型里跑得到：登记表里的目标就是轻型跑的，8 行里 65、396、657、2232 的目标是 harness 档、`--quick` 下是轻型；84、2728、3959 的目标在 checker 档，是否进轻型取决于 `--quick` 对 checker 档行的处理，我没跑 `--quick`（按定义轻型三条交回主 agent），不判。

## P4　按计划导入的正确性（丁）

判定：一致（读码；checker 档测试与变异我没跑，所有「红不红」都是推的）。读码找到三处没被用例或变异罩住的地方，记为附带发现 B1、B2、C，不改判定。

**4.1 排键的全局序号与 `my_blocks` 是不是同一套（含 ignored 的点）。** `crates/singlefs-checker-tier/src/crash_amplification.rs:7689`：「let total_blocks: u64 = plan」逐点累加 `point.blocks().len()`，`my_blocks` 在 `crates/singlefs-checker-tier/src/crash_amplification.rs:7694`：「let my_blocks: Vec<u64> = (0..total_blocks)」按这个序号取；导入那一段另起 `block_sequence`，对 `plan.points` 逐点逐块自增（不跳 ignored 点），`crates/singlefs-checker-tier/src/crash_amplification.rs:7761`：「if selected.contains(&this_block) {」跳过本机的块。两处与对照那一遍（`compare_the_gpu_and_the_cpu_conclusions` 里同样逐点 `point.blocks()` 自增）都用同一个 `PlannedCrashPoint::blocks`（`crates/singlefs-checker-tier/src/crash_amplification.rs:266`），ignored 点不管出几块两边同样数，序号不会错位。用例的流只有 3 块、1 块外人的，没有 ignored 点参与导入（ignored 点的用例是 `crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:644` 那一条，不设导入；它的断言 `…:657` 说 ignored 点一个状态都不剩，`blocks()` 为空，导入与 `my_blocks` 两边同样数不到它）；这一条靠读码，用例没钉。推翻现象：一条带 ignored 点的流上，planned 导入点名的块集合 ≠ 不在 `my_blocks` 里的块集合。

**4.2 观察者结论挂没挂全。** 导入这一段 `crates/singlefs-checker-tier/src/crash_amplification.rs:7748`：「.any(|node| node.holds_the_point(point))」，判观察者现判那一段 `crates/singlefs-checker-tier/src/crash_amplification.rs:10275`：「let is_observed = observer」同一句谓词（`holds_the_point`，`crates/singlefs-checker-tier/src/crash_amplification.rs:10123`），版本取 `observer_versions` 按观察者次序 `zip`（`crates/singlefs-checker-tier/src/crash_amplification.rs:7744`：「for (observer, observer_version) in observers.iter().zip(&observer_versions) {」，现判那一段同样 `zip`）。挂得全。用例：`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1215` 起那条挂一个观察者在发布点，钉 `…:1307`：「importing_run.imported_blocks, 2,」（CPU 判器 + 观察者各一块，总数由第二台库的 `stored_conclusion_keys().len() == 2` 另外数出）；变异 3967（漏观察者）会让它变 1 而红（推的）。

**4.3 两种判器各一把版本取得对不对。** 导入：`crates/singlefs-checker-tier/src/crash_amplification.rs:7737`：「version_of_the_point(&conclusion_versions.gpu_judge_of_each_point),」与 `crates/singlefs-checker-tier/src/crash_amplification.rs:7741`：「version_of_the_point(&conclusion_versions.cpu_judge_of_each_point),」，按 `point_index`；读回：`crates/singlefs-checker-tier/src/crash_amplification.rs:9727`：「&versions.gpu_judge_of_each_point[point_index],」同样按点取，CPU 同。给定状态表那一路用 `self.version_of(judged_by)`（`crates/singlefs-checker-tier/src/crash_amplification.rs:8855`）两种各一把。取得对。

**4.4 核对行与事实行带没带全；与旧的整库导入逐项相同吗。** 核对行：每个不归本机的块 × 两种核对代码（`crates/singlefs-checker-tier/src/crash_amplification.rs:7713`：「FactsVerifierOn::Cpu.verifier_digest(),」与 `crates/singlefs-checker-tier/src/crash_amplification.rs:7714`：「crate::crash_verify_gpu::gpu_verifier_digest(),」；没编 gpu 特性只剩 CPU 一种，读者那一侧同样只有 CPU 摘要）。事实行：按核对行值头 32 字节点名，`crates/singlefs-checker-tier/src/verdict_store.rs:1878`：「facts_digests_named.insert(facts_digest);」。路径：计划里每个点的 `path_number`。旧整库导入 `crates/singlefs-checker-tier/src/verdict_store.rs:1681`：「pub fn import_from(&mut self, other: &VerdictStore) -> Result<u64, VerdictStoreError> {」另带了三样按计划导入不带的：录入表（`record_block`）、判法版本表、旧布局的判定块。录入表本机在导入前自己录满（`crates/singlefs-checker-tier/src/crash_amplification.rs:1122`：「store.record_path(&point.path_number, &point.identity)?;」与 `crates/singlefs-checker-tier/src/crash_amplification.rs:1127`：「store.record_block(&key, count)?;」，`record_crash_points_needing` 对计划里每个块录）；判法版本表与旧布局判定块被 `unjudged_blocks`（`crates/singlefs-checker-tier/src/crash_amplification.rs:3715`）与 `ConclusionsStoredUnder::TheJudgeVersion` 那条旧路读（`crates/singlefs-checker-tier/src/crash_amplification.rs:1216`），这一趟的对账用 `unjudged_blocks_of_the_flow` 加 `TheConclusionKind`。所以流水线读回面上逐项相同；单元用例按内容读回对拍：`crates/singlefs-checker-tier/src/verdict_store.rs:3927` 起的 `importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows`，对 F 的 4 块 × 4 个（种类，版本）、2 块 × 2 种核对代码、3 条路径、事实行逐项 `assert_eq!`，并钉「按计划写下的 21 行正是 B 比 A 多出、属于 F 的那几行」。
**两种判器之外的盲区（附带发现 C）。** 旧整库导入把对方库里本机那一份的块也带过来、按计划导入只点不归本机的块：本机那一份里如果别的机器已经判过（分片权重改过之后重跑这类），旧做法复用、新做法现判。这是题面说的设计（「只导这一趟计划里不归本机那一份的块」），不算错，但与旧做法的复用面不同；给定状态表那一路同理（`crates/singlefs-checker-tier/src/crash_amplification.rs:9398`：「if !self.libraries_to_import_from.is_empty() && !self.owns_the_table(&key) {」，归本机的表不导）。

**4.5 「本库已有且相同就跳过」比的是什么，比漏会不会让旧判法的结论顶掉新的。** 比较在 `crates/singlefs-checker-tier/src/verdict_store.rs:2171`–`2183`：块（每个状态的判定向量编号，导入前已换成本库编号）、违例（块内序号加编号）、计数向量、恢复字都相同才算同，`crates/singlefs-checker-tier/src/verdict_store.rs:2183`：「if stored.judged_on != requested.judged_on {」块属性不同另报错。比较函数就是原来 `store_conclusion_block` 前半段（比较）单独抽出来的，两处共用（`crates/singlefs-checker-tier/src/verdict_store.rs:1810` 与 `crates/singlefs-checker-tier/src/verdict_store.rs:2131` 调同一个 `stored_conclusion_compared_with`；改动全文里比较那几行是未动的上下文，被拆走的是后半段「放进批、同步写」，现在在 `put_conclusion_rows_into_the_batch`，`crates/singlefs-checker-tier/src/verdict_store.rs:934`）。键是（块键，种类，判法版本），旧判法的结论在另一把版本的键下，不会进同一次比较；相同跳过不写，不同报错不写，所以任一方向都顶不掉。按编号比等价于按内容比：每个库内内容与编号一一对应（`verdict_vector_number_for_the_planned_import`，`crates/singlefs-checker-tier/src/verdict_store.rs:1992` 起）。本库没登记过的判定向量放进待写的一批，要比的块里若有这种向量，本库已存的同键结论不可能引用它，只会判成不同并报错，方向对。

**4.6 成批写入中途断了，重跑同一条命令能不能补齐、块内原子还在不在。** 一块一种结论的全部行（结论块、违例、计数向量、块属性、恢复字）与它新登记的判定向量两张表的行都放进同一个 `WriteBatch`，到字节数才换批，换批只发生在一块放完之后（`crates/singlefs-checker-tier/src/verdict_store.rs:1922`：「if pending.batch.size_in_bytes() < PLANNED_IMPORT_BATCH_BYTES {」在每块的 `put_conclusion_rows_into_the_batch` 之后才查）。所以块内原子，每个落盘的批里自带它引用的全部向量行。批写下去之后才把批里登记的向量追加进内存表（`crates/singlefs-checker-tier/src/verdict_store.rs:2055`：「self.verdict_vector_by_number.push(verdict_vector);」），编号按「已登记数 + 批内位置」发（`crates/singlefs-checker-tier/src/verdict_store.rs:2008`：「let number = u8::try_from(registered_count + position_in_the_batch)」），与追加次序一致。中途断：已落盘的批留着，重跑时那些块在本库里同键同内容，走 `already_identical` 跳过，没落盘的重导；路径与事实行比较后补。核对行与它点名的事实行不在同一批：核对行可能先落盘、事实行后落盘；重跑能补是因为 `crates/singlefs-checker-tier/src/verdict_store.rs:1878`（`facts_digests_named.insert`）排在 `crates/singlefs-checker-tier/src/verdict_store.rs:1883`：「if stored_value.as_deref() == Some(value_bytes.as_slice()) {」的跳过之前，核对行已在本库也照样点名事实。
这三处没有用例或变异罩住（附带发现 B1、B2）：B1，`PLANNED_IMPORT_BATCH_BYTES` 是 `crates/singlefs-checker-tier/src/verdict_store.rs:1024`：「const PLANNED_IMPORT_BATCH_BYTES: usize = 64 << 20;」的常量，单元用例的数据只有几百字节，只会走一批（`crates/singlefs-checker-tier/src/verdict_store.rs:4017`：「synced_writes: 1,」），换批那条分支、跨批的编号接续、「断了重跑补齐」没有一条用例；变异 3965（批长判据反过来）只在单批路径上红。B2，把 `facts_digests_named.insert` 挪到跳过之后，现有用例里本库没有对方的核对行，不会红；该变异会让断了之后落盘了核对行、没落事实行的库永远补不齐事实。推翻 4.6 的现象：造一份小批长的副本，在第二批写之前注入错误，重跑后库不等于一次跑完。我没造（checker 档）。

**4.7 给定状态表那一路判那张表时才导，会不会让该导的表被现判、或让导入失败被吞。** `crates/singlefs-checker-tier/src/crash_amplification.rs:9398` 的条件在「取已存结论」（`stored_conclusions`，其后）之前；归别的机器的表先导、再读，读到就复用；读不到、又配了要导入的库，`crates/singlefs-checker-tier/src/crash_amplification.rs:9414` 起那段返回 `GIVEN_STATES_TABLE_JUDGED_BY_NO_HOST_PREFIX` 的 `Err`，不去现判；归本机的表不导、现判。导入失败：`crates/singlefs-checker-tier/src/crash_amplification.rs:9399`：「self.import_the_table_from_the_other_hosts(&key)?;」往外传 `Err`，调用方 `crates/singlefs-checker-tier/src/crash_injection.rs:2083` 起 `unwrap_or_else(|reason| panic!(…))`，不吞。用例 `crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1409` 起那条权重 `0,1` 让每张表都归第二台，钉 `…:1487`：「(4, 4, 0, 0),」（导入 4、复用 4、现判 0、留给别人 0）并与整库导入逐张对拍；归属混合（有的表归本机）没有用例，靠变异 3968（归属反过来）：反过来之后第二台的表不导，引擎判第一张就报「导入之后库里还是没有」而红（推的）。

## P5　按计划导入的代价与钉值（丁）

判定：代价一致（逐步数读码，读写次数只随点名的项数涨）；钉值的算术另外核对过，成立；「改坏会不会红」只能推（变异行在 checker 档，我没跑）。

**5.1 读写次数逐步数。** 入口 `crates/singlefs-checker-tier/src/verdict_store.rs:1772`：`pub fn import_the_planned_rows_from(`。N 是点名项数（结论 `(2 + 挂在这个点上的观察者数) × 不归本机的块数`，核对行 `2 × 不归本机的块数`，路径 = 计划里点数）。逐项：

| 步 | 对方库 | 本库 | 出处 |
|---|---|---|---|
| 每项结论 | `read_conclusion_block`：4 次点读（结论块、计数向量、块属性、恢复字）+ 1 次只扫这一块违例前缀的迭代；没有这一项则 1 次点读就停 | `stored_conclusion_compared_with`：本库有这一块时同样 4 次点读 + 1 次前缀扫，没有时 1 次点读 + 1 次点读看恢复字旁表 | `crates/singlefs-checker-tier/src/verdict_store.rs:2170`（读）、`crates/singlefs-checker-tier/src/verdict_store.rs:2191`（旁表） |
| 违例前缀扫 | `entries_with_key_prefix`：一次 seek，`take_while` 前缀不符就停 | 同 | `crates/singlefs-checker-tier/src/verdict_store.rs:844`：「.take_while(move」 |
| 每条路径 | 1 次点读 | 1 次点读 | `crates/singlefs-checker-tier/src/verdict_store.rs:1844` |
| 每个核对行 | 1 次点读 | 1 次点读 | `crates/singlefs-checker-tier/src/verdict_store.rs:1883` |
| 每份被点名的事实 | 1 次点读 | 1 次点读 | `crates/singlefs-checker-tier/src/verdict_store.rs:1896` |
| 编号换算 | 内存里 256 格表，每种向量只按内容查一次 | 内存哈希 | `crates/singlefs-checker-tier/src/verdict_store.rs:1992` 起 |
| 写 | | 批数 = 写下字节数 ÷ 64 MiB 向上取整，一批一次同步写 | `crates/singlefs-checker-tier/src/verdict_store.rs:2046`：「.write_opt(pending.batch, &synced_write_options())」 |

整个函数里没有一处扫整张表的迭代（`grep -n 'entries_with_key_prefix' verdict_store.rs` 在这一函数的调用链里只有违例前缀那一处，前缀由块键、种类、版本定死）。所以读写次数随 N 线性，与两边库里累计的别的流、别的版本的行数无关；单次点读在 LSM 里的代价随库大小只多一个对数因子，这个「次数不涨」是按次数说的，不是按挂钟说的（调查报告没量秒数，已知欠账，我也没量）。给定状态表那一路每问一张归别人的表就导一次（`crates/singlefs-checker-tier/src/crash_amplification.rs:9398` 那处），每次 `(2 种判器) × 要导入的库数` 项，与表数线性。推翻现象：同一份对方库里额外灌几十万行别的流的行，导入的 `get`/迭代次数（RocksDB 统计）跟着涨。我没灌（没有可跑的 checker 档）。

**5.2 钉的绝对值是不是另外算出来的，改坏会不会红。**
- 21 行：`crates/singlefs-checker-tier/src/verdict_store.rs:4016`：「rows_written: 21,」与 `crates/singlefs-checker-tier/src/verdict_store.rs:4032`：「assert_eq!(written_by_plan.len(), 21, "按计划导入写下 21 行");」。我另外手算：结论块 3 块写下（F1 的 GPU、F2 的 GPU、F2 的观察者），GPU 结论带 1 条违例且带恢复字（`crates/singlefs-checker-tier/src/verdict_store.rs:3203` 起 `conclusion_record`：`recovery_words: Some(...)`）→ 结论块 + 违例 + 计数向量 + 块属性 + 恢复字 = 5 行，观察者（`crates/singlefs-checker-tier/src/verdict_store.rs:3659` 起 `observer_record`：`recovery_words: None`）4 行，合 5 + 5 + 4 = 14；B 里 F2 用到一种 A 没登记的判定向量，两张表各 1 行 = 2；路径 2（`f.0/f.1` 身份不同、`f.0/f.1/f.2` 新；`f.0` 相同跳过）；核对行 2（F1、F2 的 CPU 核对代码；F3 没判到、GPU 核对代码一行没有）；事实行 1（F 的）。14 + 2 + 2 + 2 + 1 = 21。账里其余项同样对得上：计划 3 块 × 3 种 = 9 项 = 写 3 + 已相同 1（F1 的 CPU）+ 对方库里没有 5（F1 观察者、F2 CPU、F3 三种）。这个数还被第二条路径钉住：`written_by_plan` 与 `other` 减 `entries_before` 减流 G 的行逐行相等（`BTreeSet` 比较），那是从两个库的原始键值直接求差，不经过 `PlannedImportTally`。两条路径只共用测试夹具里造的库，不共用被测的导入代码。
- 导入块数 2 与「导入 4 复用 4 现判 0」：`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1307`：「importing_run.imported_blocks, 2,」前面先从第二台库自己列出 `stored_conclusion_keys().len() == 2`；`…:1487`：「(4, 4, 0, 0),」对应 `GIVEN_STATES_TABLES_OF_THE_IMPORTED_ITEM = 4`（没配 GPU，每表一块 CPU 判器结论），并与整库导入那一份逐张对拍、列出的（块键，种类，版本）逐项相同。
- 改坏会不会红（推，没跑）：变异 `crates/mutations.tsv:3960`（漏第一项结论）→ 写 2 块、`rows_written` 变；`3961`（路径有就跳过、不比身份）→ `paths_written` 变 1；`3962`（事实行不随着导）→ `flow_facts_written` 0；`3963`（同键不比内容）→ 冲突那条用例不再报错；`3965`（批长判据反过来）→ `synced_writes` 不再是 1；`3966`、`3967`、`3968`、`3969` 各改一处，对应用例的 `imported_blocks`、复用数或逐项相同断言会变。这些行的目标（`-p singlefs-checker-tier --features verdict-store --lib` 与 `--test crash_amplification_pipeline_records_checks_and_reuses`）都在 checker 档，我复核不了。

**5.3 旧 `import_from` 留着的几处调用方还有没有随库涨的路径。** 全仓 `\.import_from(` 的调用（`grep -rn` 输出，已核）：测试 `crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:630`、`:1443`；`verdict_store.rs` 单元测试 `:3046`、`:3140`、`:3620`、`:3941`；生产代码只有一处：`crates/singlefs-checker-tier/src/crash_amplification.rs:9851`：「let imported = main_store.import_from(&self.store)?;」，在 `bring_back_into`（`crates/singlefs-checker-tier/src/crash_amplification.rs:9850`）里。被导入的 `self.store` 是 `CrossCheckLibraryBesideTheMain` 的旁库，`crates/singlefs-checker-tier/src/crash_amplification.rs:9822`：「let store = VerdictStore::create_empty(&directory)」每一趟新建空库（上一次没删干净的同名目录先删），里面只有这一趟对照那一遍分给本机、主库里还没有 CPU 结论的块。`import_from` 对它的做法：旁库的录入表、路径、事实表、核对表、判法版本表整张迭代（`crates/singlefs-checker-tier/src/verdict_store.rs:1682` 起），事实/核对/判法版本每行一次同步写（`crates/singlefs-checker-tier/src/verdict_store.rs:1695`：「let mut batch = rocksdb::WriteBatch::default();」在逐行循环里），对主库只有点读与写。所以这一处随旁库行数涨，旁库行数 ∝ 这一趟对照的块数，不随主库累计行数涨。没有别的生产调用方。风险只在：若将来有人把 `import_from` 接到一个累积库上，这些逐行同步写会随库涨；现在不是。

附带：`import_from` 的逐行同步写对事实/核对/判法版本三张表是无条件覆盖、不先比（`crates/singlefs-checker-tier/src/verdict_store.rs:1690` 起），与按计划导入「相同跳过」不同，旁库场景下无影响。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| P1 | 一致（卡上部分复核不了） | 新三条造法同 harness L2，红集断言钉 CPU 一侧，k = 2 变异只改 GPU 内核、红在 `mismatches` 那条；L1 三行第 59 批变异仍点得中；「恰好一条」的数靠第 59 批替身，卡上没复核 |
| P2 N2 | 一致 | 补进来的行过四关到水位行缺；逐条判 I-3.13 的读者改法下绿，拒统计量 7 的读者改法下红在别的成员，注释已写；台阶行之外代段 = txg 的前提成立 |
| P2 N3 | 一致 | 变异 3957、3958 在 8 条同前缀用例里各只红自己那一条（量） |
| P3 | 新取样点一致，清零再写回部分造的 | 屏障与影子账两个新用例各只被点名变异打红，老用例不红（量）；harness 档 4 行替换复跑红；84、2728、3959 复核不了 |
| P4 | 一致（读码） | 排键序号、观察者、两种判器版本、核对行、事实行与读回面对得上；附带 B1、B2（换批与断了重跑没有用例）、C（本机那一份不再复用别机的结论） |
| P5 | 代价一致，钉值成立 | 读写次数只随点名项数；21 行手算与键值求差两条路径都对；生产里唯一的旧 `import_from` 调用只吃旁库 |

附带发现：A，I-9.37（以及 I-9.36）不在 `.claude/kb/invariants.md`，I-9.22 行没有「最右那片之外 == 旋钮值」半句；B1、B2、C 见 P4。

## 没做什么

- 没起 GPU，没跑任何 checker 档测试与变异行（含 `crates/mutations.tsv` 的 3954–3956、3960–3969 以及行 84、2728、3959），没跑 `--quick` 轻型、门禁阶段、gate-lint。P1、P4、P5 里「红不红」「卡上一致不一致」全是读码或算术，已逐处标明。
- 没造 P4 的反例（小批长副本、注入错误后重跑）：要改 `PLANNED_IMPORT_BATCH_BYTES` 再编 checker 档，归攻方与主 agent。
- 没复核背景材料里 sha256 之外的东西（实现员报告里的 12/12 证红、`prove-red.sh` 等）。
- 不判别的格不判：不替主 agent 采纳或出判决。

## 要主 agent 跑的命令

- P1 卡上两件：不打变异跑 `knob_two_agrees_with_the_cpu_on` 12 条（看新三条 `mismatches=0`、`cpu_checker_red=` 墓碑 477、谱系 417、退役 513，红集断言过）；再打 3954–3956 三行看各红在 `mismatches` 那条断言。命令原样见实现员报告 `~/.local/share/singlefs/work/m3-batch60/gpu-far-left/report.md`「要主 agent 跑的命令」第 2、3 条。
- P4、P5：在 checker 档里跑 `verdict_store::tests::` 与 `crash_amplification_pipeline_records_checks_and_reuses`，并把变异 3960–3969 经轻型各跑一遍；若要关 B1、B2，加一条小批长的用例（把批长做成可注入）再补两行变异。

## 草稿与产物位置（仓外，没入库）

`~/.local/share/singlefs/work/m3-batch60-61-code-r1-sonnet/`：`tools/`（`apply.py`、`runm.sh`）、各次运行日志（`base-n2.log`、`base-n3.log`、`base-n4a.log`、`base-n4b.log`、`mut3957.log`、`mut3958.log`、`mut857.log`、`mut2755.log`、`mutNa.log`、`mutNc.log`、`mutrow65.log`、`mutrow396.log`、`mutrow657.log`、`mutrow2232c.log`）。不入库的原因：只是复核这一轮的变异量，数已写进本报告。我建的仓副本 `repo/`（24M）与编译目录 `target/`（2.9G）交回前已删（路径 `~/.local/share/singlefs/work/m3-batch60-61-code-r1-sonnet/repo`、`…/target`），要复跑：从冻结副本拷一份 `crates/`、`Cargo.toml`、`Cargo.lock` 到 `…/repo`，`tools/apply.py` 与 `tools/runm.sh` 里的路径照旧，重编约 12 秒到数分钟。
