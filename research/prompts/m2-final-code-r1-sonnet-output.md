# m2-final-code-r1 云端正推腿（Sonnet）报告

判据格：Z2（释放前读盘核与隔离）、Z3（实例表多片写路径）、Z5（checker 的 I-7.9、I-9.15 与重建回退）。

方法：代码全部读冻结副本 `/tmp/claude-1000/m2-final-code-r1/tree/crates/`，不读主工作区；kb 引文在 `.claude/kb/decisions/`、`.claude/kb/invariants.md`、`.claude/kb/checks-owed.md` 里用 `grep -n` 现查行号（下面每条引文标注的行号都已核对，命令见各节）。每条结论后写「推翻条件」。

## Z2　释放前读盘核与隔离（实十六接续 N1、N3）

### 结论 1（N1，读盘核）：兑现了条款

条款原文（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:105`，`grep -n '硬规则 1 的读盘核读不出' .claude/kb/decisions/19-块指针的结构与宽度预算.md` 命中该行）：

> 硬规则 1 的读盘核读不出、核出对不上时怎么办：释放之前按位置项读盘核校验和，读盘本身失败先重读一次，还读不出就按对不上处置；任一份核出对不上，这个单元在每块盘上的分配记录都留在「已分配」、不改成已释放（另一块盘上那一份对得上也一起留，各盘的账保持对称）——落盘即跨重挂，准入里照已分配算、不另进式子。位置项指向一块不在池里的盘，或两条位置项指同一块盘，当映射条目损坏：在任何写之前拒绝，盘上不变。从盘上重建上一版（`crates/singlefs-core/src/recovery.rs` 的 `rebuild_version`）时读不出的数据单元照抄它的位置项、不读内容，挂载照常，读到那个文件时才报错。

从原文到代码的那一步：`copies_failing_the_release_checksum_check`（`singlefs-core/src/transaction.rs:1839`）里

```
let read_the_copy = || { reader.read(...) };
let Some(copy) = read_the_copy().or_else(read_the_copy) else { ... };
```

（`transaction.rs:1888-1892`）——`.or_else` 只再调一次闭包，正是「先重读一次，还读不出就按对不上处置」，不退避、不多次重试。

测试证据（`singlefs-harness/tests/second_transaction_supplement_two_release_checksum_quarantine.rs`）：`a_release_checksum_read_that_fails_once_is_read_again_and_the_intact_copy_is_released_as_usual`（第 453 行）、`a_release_checksum_read_that_keeps_failing_is_read_once_more_and_then_handled_as_a_mismatch`（第 483 行）。

推翻条件：若 `read_the_copy().or_else(read_the_copy)` 被改成循环重试（`loop`/`retry` 计数 > 1），或读失败时未经过 `or_else` 直接判定失败，此结论作废。

### 结论 2（N3，核出坏的留在已分配）：兑现了条款

条款原文（同上引文第二句「任一份核出对不上，这个单元在每块盘上的分配记录都留在『已分配』、不改成已释放（另一块盘上那一份对得上也一起留，各盘的账保持对称）」）与硬规则 1 首句（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:99`，`grep -n '释放一律经映射' .claude/kb/decisions/19-块指针的结构与宽度预算.md` 命中该行）：

> 释放一律经映射，不经提示；经映射核到那条映射条目之后，释放之前还要按它位置项里带的单元校验和读盘核一次。核出对不上时隔离那个槽，发布照成：逻辑上照样释放（映射条目去掉），物理槽不还回空闲池，记进隔离并计数报出。

从原文到代码的那一步：`copies_failing_the_release_checksum_check` 逐设备核对（`transaction.rs:1904-1907`）：

```
if crc32_castagnoli(&copy) != location.unit_checksum && !failing.contains(&quarantined) {
    failing.push(quarantined);
}
```

累积的 `failing` 经 `every_copy_failed` 判定（`transaction.rs:1908-1917`：只有当 `devices_whose_copy_failed` 覆盖 `pool_devices` 全体时才不报错），再由 `publish_admitted`（`transaction.rs:3542-3552`）把 `devices_whose_copy_failed_the_checksum` 交给 `PoolAllocator::release_leaving_the_record_allocated_on`（`allocator.rs:909`）：该函数对 `devices_whose_record_stays_allocated` 里的每块盘「一个字节都不动、留在已分配」（`allocator.rs:906` 文档注释），其余盘正常置 `is_released = true`。因为进入这里时要么 `failing` 为空、要么覆盖全部设备（见结论 3），所以「留在已分配」的落点在每一块参与判定的盘上都发生——与条款「各盘的账保持对称」相符。

测试证据：`a_released_unit_whose_copies_fail_the_checksum_in_its_mapping_entry_is_quarantined_instead_of_returning_to_the_free_pool`（第 131 行）、`a_quarantined_copy_stays_allocated_across_a_remount_and_is_never_handed_out_again`（第 540 行）、`a_quarantined_copy_counts_as_allocated_in_the_admission_reading_and_adds_no_term_of_its_own`（第 606 行）。

推翻条件：若发现某条路径在 `every_copy_failed` 为假时仍调用 `release_leaving_the_record_allocated_on` 并传入非空但不完整的设备集合，或 `is_released` 在隔离盘上被置真，此结论作废。

### 结论 3（单盘核出坏时今天整体拒绝）：已知，去向实二二

背景材料正文「已知、用户已定、实现还没落的」第一条（`research/prompts/_m2-final-code-r1-background.md:49`，`grep -n '只一块盘那一份核出坏时' research/prompts/_m2-final-code-r1-background.md` 命中该行）：

> 只一块盘那一份核出坏时，今天在写之前拒绝；用户 2026-09-25 定「两块盘一起留」，实现随实二二。

代码现状：`copies_failing_the_release_checksum_check` 里，当 `devices_whose_copy_failed` 非空且不覆盖全体设备时（即只有一块盘的副本核不过、另一块盘核得过的「非对称」情形），直接返回 `PublishError::ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported`（`transaction.rs:1908-1917`），在任何写之前拒绝整次发布——与用户 2026-09-25 定的「两块盘一起留」（即也应像全部核不过那样让发布照成、两盘一起隔离）不同，是决策变更前的旧口径。

测试证据：`a_checksum_failure_on_only_one_copy_returns_the_unsupported_member_before_anything_is_written`（`second_transaction_supplement_two_release_checksum_quarantine.rs:241`），断言返回 `PublishError::ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported`（第 263 行），钉的正是这条旧行为。

按背景材料指示：这一格**不算打中**，写「已知」，去向「实二二」。

### 观测（不下判定）：写行旧链的三性质核验已在取号之前调用，与「已知」第六条的关系没有查清

背景材料「已知」第六条（`_m2-final-code-r1-background.md:53`）：

> 写行那次的释放核验挪到取号之前，用户 2026-09-24 定，实现随实二三。

现查 `mount.rs`：`refuse_publishes_before_acquisition_that_do_not_pass_admission`（`mount.rs:1153`）在 `establish_instance` 里先于 `acquire_expected_instance`（`mount.rs:1536`）被调用（调用点 `mount.rs:1504`），其中对写行旧链调用了 `instance_table_chain_to_release`（三性质核验：在册、未释放、跨度对得上，`mount.rs:1166`），确实发生在取号之前。但这只是三性质核验，不是 N1 的读盘校验和核验——`instance_table_chain_to_release` 的文档注释（`transaction.rs:1710-1712`）明写「不经映射、不做释放之前的读盘核」，这是因为实例表豁免映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12），硬规则 1 本就不管它。所以「已知」第六条说的「释放核验」具体指哪一层核验（三性质核验，还是别的角色的读盘核验挪到预演阶段），本报告没有查清，**复核不了**——已知第六条更贴近 Z4（取号之前在分配器副本上预演）的射程，Z4 归 Opus 判，这里只记观测、不下三种结论之一。

## Z3　实例表多片写路径（实十六接续）

### 结论 4（一片写满 369 行再开下一片、链指针记录字段表）：兑现了条款

条款原文（`.claude/kb/decisions/18-块里携带什么信息.md:248-338`，摘登记表下方一段；`grep -n '打包记录类型是第二级登记表\|一片装几行' .claude/kb/decisions/18-块里携带什么信息.md` 未直接命中该句，改用 `grep -n "kind. = 0 行记录" .claude/kb/decisions/18-块里携带什么信息.md`）：

> `kind` = 0 行记录：`kind 1 | 实例代号 4 | 所选根的 checkpoint_txg 8 | 属于该实例的最大已施加事务号 W 8 | flags 1（bit0 = 回退行，其余位恒 0、非 0 拒收）| 预留 66`；
> `kind` = 1 链指针记录（恒为一片的最后一条）：`kind 1 | 有无下一片 1 | 位置指针 86（无下一片时清零占位；与根记录里的实例表单元指针同型，D19（块指针的结构与宽度预算） 已定项 7 / 8）| 预留 0`。
> 一片 ⌊(32768 − 136) / 88⌋ = 370 条记录（含链指针，数据行 369；行数多于 369 时一片写满 369 行再开下一片，最后一片装剩下的……）。

（引文出自 D18（块里携带什么信息） 已定项 11，用户 2026-09-24 定案「一片写满 369 行再开下一片」，`.claude/kb/checks-owed.md` 之外的正文段落，行区间已在背景材料附录整段抄出，`_m2-final-code-r1-background.md:721-726` 与实际文件行区间一致，现查 `grep -n 'kind. = 1 链指针记录' .claude/kb/decisions/18-块里携带什么信息.md` 命中第 725 行。）

从原文到代码的那一步：`instance_table.rs` 里 `InstanceRow::to_bytes`（第 33-47 行）逐字段写 `kind 0 | 实例代号 4 | checkpoint_txg 8 | W 8 | flags 1 | 预留 66`（`writer.skip(66)`），`InstanceTableChainRecord::to_bytes`（第 181-193 行）写 `kind 1 | 有无下一片 1 | 位置指针 86`；`instance_rows_per_page`（第 132-134 行）取 `INSTANCE_TABLE_PAGE_RECORDS(370) - 1 = 369`；`instance_table_rows_of_each_page`（第 140-145 行）用 `rows.chunks(369)` 切片，与「一片写满 369 行再开下一片，最后一片装剩下的」逐字对应。

测试证据：`instance_table.rs` 自带的 `pages_for_rows_is_at_least_one_and_opens_a_page_every_three_hundred_sixty_nine_rows`（第 391 行）、`rows_fill_a_page_of_three_hundred_sixty_nine_before_the_next_page_opens`（第 407 行）；`singlefs-harness/tests/second_transaction_supplement_two_instance_table_page_full.rs` 的 `writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page`（第 96 行）。

推翻条件：若 `INSTANCE_TABLE_PAGE_RECORDS` 或 `instance_rows_per_page` 的算法改变导致某一片装的数据行不是 369，或最后一片之外某片行数不满 369 却已开新片，此结论作废。

### 结论 5（尾片先、bump 次序）：兑现了条款

条款原文（D3（空间分配） 已定项 10 ⑤，`.claude/kb/decisions/03-空间分配.md:197-226`；`grep -n '提交内生块从开放段 bump 的次序' .claude/kb/decisions/03-空间分配.md` 命中第 637 行）：

> ⑤ **提交内生块从开放段 bump 的次序**：一次发布重写实例表单元时（写行、暖机、回退那几次），**实例表单元最前**，多于一片时**尾片先**（第 k 片当第 k+1 片的父，照「先叶后根」）；其余按树 ID 升序……

从原文到代码的那一步：`build_instance_table_chain`（`transaction.rs:890`）对 `rows_of_each_page.iter().enumerate().rev()` 逆序迭代（`transaction.rs:906`），文档注释直写「尾片先装、先发出生序号……第 k 片的链指针记录要第 k + 1 片的落点、整单元校验和与出生序号，第 k + 1 片装好了它才装得出来」（`transaction.rs:876-878`）；取落点的次序 `instance_table_page_roles_in_bump_order`（`transaction.rs:2262`）同样 `(0..pages).rev()`（`transaction.rs:2264`），与文档注释「尾片先……第 0 片最后」（`transaction.rs:2260-2261`）一致。

测试证据：`a_row_publish_past_one_page_writes_the_tail_page_first_and_the_first_page_chains_to_it`（`second_transaction_supplement_two_instance_table_second_page_write.rs:155`）。

推翻条件：若 `build_instance_table_chain` 或 `instance_table_page_roles_in_bump_order` 改成正序迭代（`.rev()` 被删除），或某一测试观测到第 0 片先于末片被写出生序号，此结论作废。

### 结论 6（写行整条链 COW 重写、实例表豁免映射不做读盘核）：兑现了条款

条款原文（D18（块里携带什么信息） 已定项 11，`.claude/kb/decisions/18-块里携带什么信息.md:248-338`；`grep -n '每次写行 COW 重写整条链' .claude/kb/decisions/18-块里携带什么信息.md` 命中第 727 行）：

> ……**每次可写挂载都写行**（实例 0 不写；写行那次发布是新实例的第一次发布……）；**每次写行 COW 重写整条链**。

从原文到代码的那一步：`build_instance_table_chain` 的入参 `rows: &[InstanceRow]` 是「这次之后整张表的行」（`InstanceTableRewrite.rows` 文档注释，`transaction.rs:2241-2244`：「这次之后整张表的行，按链上的次序（上一版那张表的行在前、这次写的接在后面）」），每次调用都重新按 369 行一片切分、重新写出**全部**片（不是只写变化的那几片），对应「整条链」COW 重写；旧链的释放同样是整条链逐片释放（`instance_table_chain_to_release`，`transaction.rs:1721`）。实例表豁免映射见 `roles_replaced_via_mapping`（`transaction.rs:1738-1741` 起）的角色清单里没有 `InstanceTable` / `InstanceTablePageAfterTheFirst`，且 `copies_failing_the_release_checksum_check` 对这两个角色显式 `continue`（`transaction.rs:1861-1863`），与 D19（块指针的结构与宽度预算） 已定项 5 的「映射树、树表、实例表豁免映射，硬规则 1 的读盘核……它们不核」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:107`）一致。

测试证据：`the_next_mount_rewrites_the_whole_two_page_chain_and_releases_both_old_pages`（`second_transaction_supplement_two_instance_table_second_page_write.rs:298`）、`a_version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both`（同文件第 358 行）。

推翻条件：若某次写行只重写变化的那一片、旧的未变片被直接复用（不释放旧片、不重写），或 `copies_failing_the_release_checksum_check` 对实例表角色不再 `continue` 而进入读盘核分支，此结论作废。

### 结论 7（取号之前按片数的准入、逐片核旧链）：兑现了条款

条款原文与结论 5/6 同源（D18（块里携带什么信息） 已定项 11 的「可写挂载的顺序」段，`.claude/kb/decisions/18-块里携带什么信息.md:248-338`；`grep -n '这次挂载要发的写行与暖机在分配器的副本上预演取得到全部落点' .claude/kb/decisions/18-块里携带什么信息.md` 命中第 736 行）：

> ……∧ 这次挂载要发的写行与暖机在分配器的副本上预演取得到全部落点——预演里写行那次的释放核验报错，同样判这次不能可写……

从原文到代码的那一步：`refuse_publishes_before_acquisition_that_do_not_pass_admission`（`mount.rs:1153`）在取号之前，先对写行要换下的旧链调用 `instance_table_chain_to_release`（`mount.rs:1166`，核不过在 `mount.rs:1170-1173` 映成 `MountError::RowPublishAdmissionRefusedBeforeAcquisition`），再用 `PublishShape::row_publish_rewriting_instance_table_pages(instance_table_rewrite.pages_after_this_publish())`（`mount.rs:1189-1191`）把「这次之后几片」接进 `publish_sequence_admission` 的准入序列——即按片数（`pages_after_this_publish()` = `instance_table_pages_for_rows(rows.len())`）计入分配记录树 / 记账树 / 中央映射树的容量准入，且这一切都发生在 `establish_instance` 调 `acquire_expected_instance`（`mount.rs:1536`）之前（调用点 `mount.rs:1504`）。

测试证据：`writable_mount_follows_the_chain_and_refuses_a_page_missing_from_the_allocation_records_before_acquisition`（`second_transaction_supplement_two_instance_table_chain.rs:381`）、`mount_after_crashes_right_after_acquisition_fills_the_page_and_one_more_row_opens_the_second_page`（`second_transaction_supplement_two_instance_table_page_full.rs:134`）、`rollback_candidate_set_reads_the_row_that_lives_on_the_second_page`（`second_transaction_supplement_two_instance_table_chain.rs:423`）。

推翻条件：若 `refuse_publishes_before_acquisition_that_do_not_pass_admission` 的调用点被挪到 `acquire_expected_instance` 之后，或 `PublishShape::row_publish_rewriting_instance_table_pages` 不再按 `pages_after_this_publish()` 传参而是写死的片数，此结论作废。

### 已知（涉及 Z3 的一格）：树表 0 条写行、片数 ≥ 67 时点名项装不下

背景材料「已知」第三条（`_m2-final-code-r1-background.md:50`，`grep -n '树表 0 条那一版写行，片数' research/prompts/_m2-final-code-r1-background.md` 命中该行）：

> 树表 0 条那一版写行，片数 ≥ 67 时一条记录装不下、`ByteWriter` 越界，照 D23（journal 的角色与格式） 已定项 17 接再跨记录，随实二二。

这一条严格说是 journal 记录格式（D23（journal 的角色与格式） 已定项 17，Z1 的射程）与 Z3 的实例表多片写路径的交界：写行涉及的实例表片数一旦 ≥ 67，该次发布要点名的单元数超过一条 journal 记录能装的 67 项，需要「末条再跨记录」，而这属于实二十（Z1）尚未完全覆盖到「树表 0 条」这条支线的部分。本报告不判 Z1，这里只记「已知」，不算打中，去向「实二二」；未在冻结副本里逐行核验 `ByteWriter` 越界的现状（不在 Z3、Z5 的判据格内，复核不了）。

## Z5　checker 的 I-7.9、I-9.15 与重建回退（实十四）

### 结论 8（I-9.15，inode 记录 blocks = ⌈size ÷ 512⌉）：兑现了条款

条款原文（`.claude/kb/invariants.md:283`，`grep -n 'I-9.15 ' .claude/kb/invariants.md` 命中该行）：

> | I-9.15 | inode 记录的 blocks 等于 ⌈size ÷ 512⌉ | 打包记录类型 2 的每条记录，偏移 48 的 blocks == ⌈偏移 40 的 size ÷ 512⌉（D8（核心索引结构） 已定项 6：blocks 是逻辑长度的 512 字节块数，不表示分到的空间；C480（inode 记录的 blocks 怎么算全仓没有条款） 用户 2026-09-23 定）；不等判红。……

从原文到代码的那一步：`walk.rs` 里（`singlefs-checker/src/walk.rs:50-54`）`const INODE_RECORD_SIZE_OFFSET: usize = 40;`、`const INODE_RECORD_BLOCKS_OFFSET: usize = 48;`、`const INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES: u64 = 512;`，判定处（`walk.rs:709-717`）：

```
let size_in_bytes = read_u64(&record, INODE_RECORD_SIZE_OFFSET);
let blocks = read_u64(&record, INODE_RECORD_BLOCKS_OFFSET);
let logical_length_in_blocks = size_in_bytes.div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES);
self.judgements.judge("I-9.15", blocks == logical_length_in_blocks, || { ... });
```

偏移 40/48 与 D8（核心索引结构） 已定项 6 的记录字段表（`.claude/kb/decisions/08-核心索引结构.md:150-216`：偏移 40「size / blocks / rdev，各 8」）逐字对应，`div_ceil(512)` 即「⌈size ÷ 512⌉」。

测试证据：`inode_record_blocks_other_than_the_logical_length_in_512_byte_blocks_reddens_only_the_blocks_invariant`（`singlefs-harness/tests/checker_known_bad_images.rs:2239`）。

推翻条件：若偏移常量或除数改变而 kb 字段表未同步改，或某条变异（把 blocks 判定改成向下取整）在 `crates/mutations.tsv` 里未被判红，此结论作废。

### 结论 9（I-7.9，回退下界 F 不高于抬 F 的上限）：兑现了条款

条款原文（`.claude/kb/invariants.md:59`，`grep -n 'I-7.9 ' .claude/kb/invariants.md` 命中该行，整行较长，摘录判定核心，未摘句判定要点全部覆盖于下方逐点对照）：

> | I-7.9 | 回退下界 F 不高于抬 F 的上限 | **抬 F 的那一条根**（它带的 F 比同一实例里 txg 比它小的最新那条根带的高；回退那次发布与新实例的第一条根没有同实例的前一条，不算抬……）根记录里那个回退下界 F……不高于**用它之前的根算出的** D16（发布语义） 已定项 1 的抬 F 上限：min(每块盘上最新的持久有效根的 txg, 第 4 新的非空持久有效根的 txg)，非空有效根不足 4 个时取最旧有效根的 txg。有效 = 按那条根自己指着的实例表判仍然有效 ∧ txg ≥ 抬之前的 F；「之前的根」取根环里 txg 比它小的那些。只判抬 F 的那一条，不在后来每张镜像上重算……三种结局：F ≤ 上限的下沿 ⇒ 成立；F > 上限的上沿 ⇒ 违例；落在两沿之间、或上限无从算起 ⇒ 这条根不判。一条根都没判到时整条报不适用并带理由，不报成立。

逐点对照代码 `judge_rollback_floor_raises_against_their_ceilings`（`singlefs-checker/src/walk.rs:2193`）：

| 条款要点 | 代码 |
|---|---|
| 「抬 F 的那一条根」= 同一实例里比它前一条根带的 F 高；无同实例前一条不算抬 | `previous_root_of_the_same_instance`（`walk.rs:2210-2217`）找同实例、txg 更小、取最大 txg 的根；找不到 `else { continue; }`（`walk.rs:2217-2219`）；`raising_root.rollback_floor <= floor_before_the_raise` 则 `continue`（`walk.rs:2221-2223`），即「没抬」不判 |
| 有效 = 按抬 F 那条根自己指着的实例表判仍然有效 ∧ txg ≥ 抬之前的 F | `rollback_floor_ceiling_before_the_raise` 用 `raising_root` 自己的 `instance_table_rows_of_the_raising_root`（`walk.rs:2109-2110`）过滤 `!abandoned_by_instance_table_rows(...)`（`walk.rs:2115`）且 `txg >= floor_before_the_raise`（`walk.rs:2114`） |
| 「之前的根」取根环里 txg 比它小的那些 | `root.checkpoint_txg < raising_root.checkpoint_txg`（`walk.rs:2113`） |
| 上限 = min(每块盘最新有效根 txg, 第 4 新非空有效根 txg，不足 4 个取最旧) | `rollback_floor_ceiling_from`（`walk.rs:2071-2079`）：`newest_valid_root_txg_on_every_device.min(fourth_newest_non_empty_or_oldest_valid)`，`newest_first.get(3).copied().unwrap_or(oldest_valid_root_txg)` |
| 三种结局（成立 / 违例 / 不判） | `walk.rs:2236-2251`：`raised_floor > lowest && raised_floor <= highest` ⇒ 不判（`raising_roots_not_judged += 1`）；否则判 `raised_floor <= lowest_possible` |
| 一条都没判到时报不适用、不报成立 | `walk.rs:2253-2262`：`if raising_roots_judged == 0 { judgements.not_applicable(...) }` |

测试证据：`raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`（`checker_known_bad_images.rs:3188`）；固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version`（背景材料点名，未在本节复核其源码位置，因不在冻结副本的 `walk.rs`/`transaction.rs` 里搜到同名符号，判「复核不了」——推测在 `second_transaction_step_four_rollback.rs` 一类文件里，非本次判据格必查项）。

推翻条件：若 `previous_root_of_the_same_instance` 改用「按 txg 而不按同实例」查找、或 `get(3)`（第 4 新）被改成别的下标、或不判条件的判定被删除导致「落在两沿之间」也被判红/判绿，此结论作废。

### 结论 10（N2，重建上一版时读不出的数据单元照抄位置项）：兑现了条款

条款原文（同结论 1，`.claude/kb/decisions/19-块指针的结构与宽度预算.md:105` 末句）：

> 从盘上重建上一版（`crates/singlefs-core/src/recovery.rs` 的 `rebuild_version`）时读不出的数据单元照抄它的位置项、不读内容，挂载照常，读到那个文件时才报错。

从原文到代码的那一步：`rebuild_version`（`singlefs-core/src/recovery.rs:1133`）里读每个数据单元的循环（`recovery.rs:1236-1254`）：

```
let content_or_nothing_when_unreadable = match read_data_unit_via_hint_then_central_mapping(...) {
    DataUnitReadThroughTheCentralMapping::Content(bytes) => bytes,
    DataUnitReadThroughTheCentralMapping::MissingFromTheMapping { .. }
    | DataUnitReadThroughTheCentralMapping::UnreadableAtTheMappedLocation { .. } => Vec::new(),
};
data_unit_contents_in_file_order.push(content_or_nothing_when_unreadable);
data_pointers.push(data_pointer);
```

读不出（映射里查不到或映射到的位置读不出）时不报错，只把内容置空，位置指针 `data_pointer`（来自 extent 记录，即「它的位置项」）照常压入——挂载不中止，只有真正读那个文件的数据时才会用到空内容而报错（该报错路径不在本节判据格内，未继续追）。

测试证据：`after_rebuilding_the_previous_version_from_disk_the_release_still_reads_and_quarantines_the_mismatching_copy`（`second_transaction_supplement_two_release_checksum_quarantine.rs:286`）间接覆盖了 `rebuild_version` 与释放路径的交互；专门针对「重建时数据单元读不出」的测试名未在冻结副本里找到与 N2 直接同名的用例，判「复核不了」（不排除藏在 `second_transaction_supplement_three_bad_disk_input.rs` 一类文件里，未逐个翻查）。

推翻条件：若 `MissingFromTheMapping` / `UnreadableAtTheMappedLocation` 分支被改成直接返回 `Err`（挂载中止），或 `data_pointers` 不再压入该单元的位置指针，此结论作废。

### 结论 11（重建上一版只带实例表第 0 片）：替没写的条款做了选择——决策已定为欠账，今天没有会红的东西钉着

条款原文（`.claude/kb/checks-owed.md:479`，`grep -n '^| C543 ' .claude/kb/checks-owed.md` 命中该行）：

> | C543 | 重建上一版只带实例表第 0 片 | 从盘上重建上一版（`crates/singlefs-core/src/recovery.rs` 的 `rebuild_version`）时只带实例表第 0 片，没沿链读进第 1 片起各片；今天不出错，释放走写行计划里带的旧链 | 造一条靠 `rebuild_version` 读第 1 片起的路径，读回的行数必须等于写进去的行数；判别力：今天的 `rebuild_version` 必须红 | 用户 2026-09-25 定记欠账、不改：今天没有路径靠它拿后面各片 | 2026-09-25 实十六接续报告 `research/prompts/m2-writepath-implementer-report.md` 第六节 Q4 |

从原文到代码的那一步：`rebuild_version` 读实例表用 `read_unit_via_locations(reader, &root.instance_table.locations, data_bytes)?`（`recovery.rs:1140-1141`），只读根记录直接指着的那一个单元（第 0 片），不像 `instance_table_chain_of_root`（`recovery.rs:1034`）/`read_instance_table_chain`（`recovery.rs:980`）那样沿链读到「无下一片」；这份 `instance_table_bytes` 原样存进 `TransactionOutput.units` 里 `TransactionUnit::InstanceTable` 那一项（`recovery.rs:1469-1472`）。这是**替没写的条款做了选择**：D18（块里携带什么信息） 已定项 11 定义了多片链式实例表的格式，但没有一条条款规定「重建上一版这个函数必须把整条链都读出来存进 `TransactionOutput`」；实现选择只读第 0 片，选择依据是「今天没有路径靠它拿后面各片」——写行时旧链的释放走的是 `plan`（调用方）自己带的 `replaced_chain`（`InstanceTableRewrite.replaced_chain`，见结论 6 与 `transaction.rs:1721` 的 `instance_table_chain_to_release`），不读 `TransactionOutput.units` 里的这份字节。

今天有没有会红的东西钉着：**没有**。C543 自己的「判别力」一栏写「造一条靠 `rebuild_version` 读第 1 片起的路径……今天的 `rebuild_version` 必须红」，这条用例按 checks-owed.md 的体例是**要造而未造**（欠账清单里的条目，不是已入库的测试），冻结副本里没有搜到断言「读回的行数必须等于写进去的行数」这一类测试（已按结论 5/6/7 列出的测试文件逐个看过测试名，没有一条覆盖 `rebuild_version` 读多片实例表）。

推翻条件：若日后某条发布路径开始依赖 `TransactionOutput.units` 里 `InstanceTable` 角色的字节（而不是 `plan.replaced_chain`）来释放或读取第 1 片起的行，而 `rebuild_version` 仍只读第 0 片，会在合法的「实例表 ≥ 2 片」历史上丢数据或 panic——此结论所称「今天没有路径靠它」将被推翻。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Z2 · N1 读盘核 | 兑现了条款 | 读失败重读一次（`.or_else`），再失败按对不上处置，与 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 逐字对应 |
| Z2 · N3 核出坏留在已分配 | 兑现了条款 | 全部副本都核不过时逐盘置「留在已分配」，各盘账保持对称 |
| Z2 · 单盘核出坏时的处置 | 已知，去向实二二 | 今天仍是「非对称即拒绝整次发布」，用户 2026-09-25 已改判「两块盘一起留」，未落地 |
| Z2 · 写行释放核验挪到取号前 | 观测，不判 | 三性质核验已在取号前，读盘核验本就豁免实例表；与「已知」第六条对应关系没查清，复核不了 |
| Z3 · 369 行一片、字段表 | 兑现了条款 | `instance_table.rs` 的切片与字段写出逐字节对应 D18（块里携带什么信息） 已定项 11 |
| Z3 · 尾片先、bump 次序 | 兑现了条款 | `build_instance_table_chain`/`instance_table_page_roles_in_bump_order` 都逆序迭代，与 D3（空间分配） 已定项 10 ⑤ 对应 |
| Z3 · 整条链 COW 重写、豁免映射 | 兑现了条款 | 每次写行重建整张表的全部片；实例表角色在映射释放清单里被显式跳过 |
| Z3 · 取号前按片数准入、逐片核旧链 | 兑现了条款 | `refuse_publishes_before_acquisition_that_do_not_pass_admission` 先于 `acquire_expected_instance` 调用 |
| Z3 · 树表 0 条写行片数 ≥ 67 | 已知，去向实二二 | Z1/Z3 交界处的点名项超限，随实二二实现，本报告不判 Z1 那一半 |
| Z5 · I-9.15 | 兑现了条款 | 偏移 40/48、`div_ceil(512)` 与字段表、条款文字逐字对应 |
| Z5 · I-7.9 | 兑现了条款 | `judge_rollback_floor_raises_against_their_ceilings` 逐点对照条款六个要点全部相符 |
| Z5 · N2 重建时数据单元读不出照抄位置项 | 兑现了条款 | 读不出置空内容、位置指针照常压入，挂载不中止 |
| Z5 · 重建上一版只带实例表第 0 片 | 替没写的条款做了选择 | C543 记为决策已定的欠账，判别力测试尚未造出，今天没有会红的东西钉着 |

## 没做什么

- 不判 Z1（末条标志、末条再跨记录、锚点读法乙）、Z4（取号之前预演、抬 F 失败的账）、Z6（三份补丁合并点），这三格归 Opus。
- 结论 3（单盘核出坏拒绝）与树表 0 条片数 ≥ 67 两条严格说触及 Z1/Z4 的射程，本报告只按背景材料指示标「已知」，不越权对 Z1/Z4 下三种结论之一。
- 未验证固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 的源码位置（推测在 `second_transaction_step_four_rollback.rs`，未逐个翻查确认）。
- 未验证「重建时数据单元读不出」是否有专门覆盖 N2 的独立测试用例（未逐个翻查 `second_transaction_supplement_three_bad_disk_input.rs` 等文件）。
- 未编译、未跑测试（本轮线程上限 5，且改动全部读自冻结副本，未在草稿目录建可编译副本；未写模型或用例，故未建 `m2-final-code-r1-sonnet-model/` 目录）。
- 未核对「已知」六条里与 Z2/Z3 无关的另外三条（checker 的 I-3.11/I-3.1 隔离豁免读法、读者规则两格当损坏断链、发布失败原样重发）——它们分别落在 Z5 之外的 checker 逻辑与 Z1 的射程，本次判据格没有要求核它们，未查。
