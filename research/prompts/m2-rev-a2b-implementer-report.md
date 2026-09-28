# 实审 A2b 报告：审阅第 27、36、37 条与「journal 环短于三条记录 mkfs 拒」

日期：2026-09-27 开工。规格 `/tmp/claude-1000/impl-rev-a2b/spec.md`；派发提示要求不改 `crates/mutations.tsv`，要追加的变异行在 `/tmp/claude-1000/impl-rev-a2b/mutations-append.tsv`（19 行，六段，与表同格式），另有一行要整行替换的在 `/tmp/claude-1000/impl-rev-a2b/mutations-replacements.tsv`（1 行，见「要主 agent 处理的」第 2 条）。

## 结论

- 四件都照规格做了，每件先写会红的用例、在改前的代码上看着它红，再改：新测试文件 12 条用例，改前（探针版，见下）跑了其中 10 条，9 条红、1 条是保留分流的对照（改前改后都绿）；另 2 条是后补的保留行为对照（世代号相同取偏移小的、槽 0 可择时照旧借槽距不逐档试），改前的代码上按构造也绿，靠变异证红。
- 改后 12 条全绿（主工作区与证红副本各跑一次）；`singlefs-core --lib` 127 条全绿。
- 19 行变异逐条经 `research/scripts/prove-red.sh` 证红，19/19 抓到；每条新用例至少被一行证过。
- 要主 agent 处理的三件（详见「要主 agent 处理的」）：
  1. A2a 的测试 `core_review_geometry_back_chain_and_empty_inode.rs` 第 231 行那条断言「一条记录长的环放行」按这次的定案**必然红**（实跑已红），那份文件不在我的文件单里，没改；改法是一行。
  2. 表里第 933 行（实审 A2a 第 37 条那条变异）在这次改动之后**抓不到了**（实跑：没红）；我把它改点名到新用例上的整行替换放在 `mutations-replacements.tsv`，实跑抓到。
  3. 门禁 74 号在主工作区判红（随机历史 2 条）；在副本里带 / 不带我的改动各跑一次，红的一样，不是这次改动带来的（「要主 agent 处理的」第 3 条）。
- 推翻条件：主工作区（等别的会话把 `allocator.rs`、`admission.rs` 改到 clippy 过之后）跑 `cargo test -p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search` 有一条红，或 19 行变异里有一条在门禁 59 号上没红，本报告的「做完」就不成立。

## 这一轮写过的文件

- `crates/singlefs-core/src/recovery.rs`（第 27、36、37 条）
- `crates/singlefs-core/src/mounted_read.rs`（第 27 条）
- `crates/singlefs-core/src/make_filesystem.rs`（环短于三条记录）
- `crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs`（新建，12 条用例；还没被 git 跟踪，不在下面的 `git diff --stat` 里）
- `crates/mutations.tsv`：**没动**。追加的 19 行名字全以「实审 A2b」起头：「第 27 条：」6 行、「第 36 条：」2 行、「第 37 条：」9 行、「环短于三条记录：」2 行，与主工作区 `crates/mutations.tsv` 现有的名字逐个比过，不撞。
- 我这三份源文件改动之前的样子存在 `/tmp/claude-1000/impl-rev-a2b/orig/`（开工时从主工作区拷的，那时已带着别的会话没提交的改动）；我的改动 = 主工作区现状对它的 diff，逐文件 `diff -u` 在 `/tmp/claude-1000/impl-rev-a2b/my-diff-*.patch`。改了多少行（`grep -c '^[-+][^-+]'`，定稿之后数的）：recovery 226、mounted_read 23、make_filesystem 28；新测试文件 827 行。

`git diff --stat -- crates litmus` 原样附在文末一节（也存在 `/tmp/claude-1000/impl-rev-a2b/git-diff-stat.txt`）。

## 每条怎么改

### 第 27 条（树表同一种树两条：判损坏）

- `recovery.rs` 新加 `tree_table_entries_each_kind_at_most_once`（第 1379 行起，`pub(crate)`）：逐条 `TreeTableEntry::parse`，同一个 `kind` 第二次出现就交回 `TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice`，解不开交回 `EntryMalformed`。`From<TreeTableEntriesRefusal> for RecoveryFailure` 把前者换成 `UnitMalformed { what: TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE }`（新 `pub const`，第 1348 行，字面值沿用 `user_visible_tree_root_pointers` 原有的那一句「树表里同一种树有两条」），后者换成原来各处报的 `UnitMalformed { what: "树表条目" }`。
- 规格点名的五个读者都改成经它取条目：`allocation_records_under_root`（第 1151 行）、`rebuild_version`（第 1454 行）、`user_visible_tree_root_pointers`（第 1888 行；原来它自己那段 `replace(..).is_some()` 的判重删掉，改成直接赋值，因为上面已判过）、`walk_to_file`（第 2648 行）、`mounted_read::open_pool_for_read`（`mounted_read.rs` 第 402 行：解不开照旧报 `OpenPoolForReadFailure::TreeTableEntryMalformed`，同一种两条报 `OpenPoolForReadFailure::Walk(同一个 UnitMalformed)`，与另四处报的是同一个值）。
- 没改的：`tree_table_entry_count`（只数条数、不按种类取条目，发布路径拿它判「是不是 0 条」）。`user_visible_tree_root_pointers` 里「种类没登记」那一判照旧，次序变成先判重、后判没登记（`# Errors` 写明了）。

### 第 36 条 recovery 那一处（txg 加一溢出）

- `replay_journal` 里 `CheckpointTxg(root.checkpoint_txg.0 + 1)`（第 2161 行起）改成 `checked_add(1)`，`None` ⇒ `UnitMalformed { what: CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR }`（新 `pub const`，第 2102 行），在施加任何一条记录之前返回；`recover` 与 `mount_read_only` 原样把它交出去（`effective_root` 为 `None`，文档跟着补了）。
- 成员用 `UnitMalformed` 没新开：`RecoveryFailure` 在 `crates/singlefs-harness/src/history.rs` 的 `recovery_failure_member` 里被穷举 `match`，那份文件不在我的文件单里，加成员会让它编不过（见「要主 agent 处理的」第 4 条）。
- 同一个函数里另外三处 `+ 1` 没改，写不出会溢出的输入：`counter + 1`（第 2158、2220 行）的 `counter` 是 `JournalRecord::parse` 从盘上 6 字节读的（`journal.rs` 此刻第 291 行 `get_six_byte_unsigned`；别的会话在改这份文件，行号会漂），最大 2⁴⁸ − 1；第 2206 行 `u64::from(ordinal_within_publish.0) + 1` 是 `u32` 放宽到 `u64` 再加。`mount.rs`、`transaction.rs` 里同类的归 A2c，没碰。

### 第 37 条全池槽 0 都自证不过那一格（逐档试槽距）

- `choose_system_configuration`（`recovery.rs` 第 722 行那一处）：自己的槽 0 与池里别的盘的槽 0 都给不出槽距时，不再按 4096 读一次，改调新函数 `slot_one_found_by_trying_every_slot_spacing_the_format_allows`（第 620 行起）。
- 逐档：槽距从 4096（`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`）到根环基址（`root_ring::region_start(0)`，1 MiB）减 4096，按 512 字节一档（新常量 `SLOT_SPACING_SEARCH_STEP_BYTES`，第 600 行；512 的来处写在常量的文档里：槽距 = 4096 向上取整到 io_min 的整数倍，io_min 是逻辑块宽的整数倍），共 2033 档、每档每盘读一槽。
- 每档的分流与槽 0、槽 1 同一个（`mountable_slot_or_refusal`）：可择、且自己记的 `fixed_structure_slot_spacing` 等于这一档 ⇒ 候选；几个候选取世代号最大的（D22（单元原子性怎么合成） 已定项 16「择槽取校验和过且世代号最大的」，逐盘计），相同取偏移小的（与原来槽 0、槽 1 世代号相同取槽 0 同一个次序）；一个候选都没有而试到过 incompat 位图不认识的槽 ⇒ 交回按偏移升序第一个那一槽的位图（保留「布局不认识」那一报；改前按 4096 读到的就是它）；自证得过而 S 越界 ⇒ 整池拒（与槽 0、槽 1 同一个判法）。
- 各盘择到的之间照旧只由原有那一判比 fsid 与设备数：对不上报 `SystemConfigurationsDisagree`。
- 风险照定案写进了函数文档（第 611 行起那段「风险：」）：同一块盘上更早一次 mkfs 用别的槽距写下的槽 1，mkfs 不清，它与本池的槽 1 分不出；世代号更大就择到它（fsid 不同 ⇒ 整池报对不上；fsid 相同 ⇒ 按旧池几何挂）。用例 `the_search_takes_the_highest_generation_even_when_it_is_a_stale_slot_of_an_earlier_make_filesystem` 把这一格钉成「择到旧槽」，名字里写明了。

### 环短于三条记录（mkfs 拒）

- `make_filesystem.rs` 新成员 `MakeFilesystemError::JournalRingHoldsFewerRecordsThanTheSafetyFactor { ring_bytes, minimum_ring_bytes }`（第 123 行），`check_geometry` 在「环短于一条记录」那一判之后、环长上界之前判 `journal_in_flight_record_limit(ring_bytes) == 0`（第 277 行：直接判恢复用的那个在飞上限，不另写一遍「环槽数 < F」），`minimum_ring_bytes` = `JOURNAL_SAFETY_FACTOR` × `JOURNAL_RECORD_BYTES` = 12288。在任何写之前拒。
- 原有的 `JournalRingShorterThanOneRecord` 留着（`journal::record_offset` 的 `expect` 消息点名它），短于 4096 的环照旧报它。

## 每条新测试：改前红在哪、改坏哪一行 → 哪条断言红

测试文件 `crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs`（名字不含 layer0，全是内存稀疏盘上的快用例，整个二进制 0.03 秒）。

**改前**：开工时把主工作区拷成副本（`rsync -a --exclude target --exclude .git`，那时我这三份源文件与 `orig/` 逐字节相同，`cmp` 核过），放进一份探针版测试（`/tmp/claude-1000/impl-rev-a2b/probe-version-of-the-test.rs`：改前没有的两个 `pub const` 照改后要给的字面值写成本地常量；环那一条只判 `is_err()`，因为改前没有那个成员）。那时只写了前 10 条。结果 `test result: FAILED. 1 passed; 9 failed`，日志 `/tmp/claude-1000/impl-rev-a2b/prefix-run.log`。红在哪（原样摘自日志）：

| 用例 | 改前红在 |
|---|---|
| `a_tree_table_carrying_one_kind_of_tree_twice_is_refused_as_damaged_by_every_reader` | 第 315 行（探针版行号）「分配记录树两条（逐字节相同）：五个读者都判树表损坏」 left: 五个读者全 `Ok(())`（对照那一段——原样重建的树表与写者写的逐字节相同、五个读者都读得下去——改前就过了） |
| `a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking` | `crates/singlefs-core/src/recovery.rs:2039:56` 「attempt to add with overflow」（改前行号） |
| `slot_one_is_found_by_trying_every_spacing_when_no_slot_zero_in_the_pool_verifies` | 「两块盘的槽 1 都在槽距 8192 处: NoValidSystemConfiguration { first_device_with_no_valid_system_configuration_slot: DeviceIdentity(0) }」 |
| `a_slot_spacing_that_is_a_multiple_of_512_bytes_only_is_found_too` | 「两块盘的槽 1 都在 4608 处: NoValidSystemConfiguration { … DeviceIdentity(0) }」 |
| `a_self_verifying_slot_whose_recorded_spacing_is_not_its_own_offset_is_not_taken` | left: `(8192, 9)` right: `(8192, 1)` |
| `the_search_takes_the_highest_generation_even_when_it_is_a_stale_slot_of_an_earlier_make_filesystem` | left: `(4096, 1)` right: `(8192, 5)` |
| `devices_whose_highest_found_slots_disagree_on_the_filesystem_identifier_refuse_the_pool` | 「盘 0 择到另一个池的、盘 1 择到本池的：fsid 对不上」 left: `None` right: `Some(SystemConfigurationsDisagree)` |
| `the_search_tries_the_largest_spacing_whose_slot_one_ends_at_the_root_ring_base` | left: `(DeviceIdentity(0), 4096, 1)` right: `(DeviceIdentity(0), 1044480, 2)` |
| `a_journal_ring_holding_fewer_records_than_the_safety_factor_is_refused_before_any_write` | 「环 4096 字节装不下三条记录：None」 |
| `the_search_keeps_the_incompat_and_slots_per_region_refusals_of_slot_one` | 改前绿（对照：保留下来的两种分流） |

后补的两条（`a_device_whose_slot_zero_fails_borrows_the_spacing_another_slot_zero_records_instead_of_searching`、`slots_of_equal_generation_found_by_the_search_resolve_to_the_smaller_offset`）钉的是改前就有、改后要保住的次序，按构造改前也绿（改前槽 0 可择时借槽距；改前只试 4096），没在改前的副本上跑，靠下表第 19、20 行证红。

**改后变异证红**：副本 `/tmp/claude-1000/impl-rev-a2b/work-copy`（就是上面那份副本，换进我这 4 份文件；与主工作区逐字节相同，`cmp` 核过；它自己的 target）。19 行追加进副本自己的 `crates/mutations.tsv`，经 `bash research/scripts/capped.sh 4 bash research/scripts/prove-red.sh --copy <副本> --memory 8G singlefs-harness <名…>` 跑，三批。基线红集为空：第一批基线 `test result: ok. 10 passed; 0 failed`（`prove-red-logs/baseline.log`），第二批 `ok. 11 passed`，第三批 `ok. 12 passed`。第一批 `✓ 点名 18 条：跑了 18 条，跳过 0 条，跑的都抓到了`，第二批 `✓ 点名 2 条：跑了 2 条，跳过 0 条，跑的都抓到了`（其中一行是副本专用的 release 版，不进交回的变异行）。

| # | 变异（名字省去前缀「实审 A2b」） | 必须红的用例 → 红在哪一条断言 | 同一个二进制里同时红的 |
|---|---|---|---|
| 1 | 第 27 条：树表条目不判同一种树两条 | 树表那条 → 第 316 行 assert_eq：五个读者全 `Ok(())` | 无 |
| 2 | 第 27 条：走读不经共用判定 | 同上：只有 `walk_to_file: Ok(())`，另四个照旧报损坏 | 无 |
| 3 | 第 27 条：重建上一版不经共用判定、取首条 | 同上：只有 `rebuild_version: Ok(())` | 无 |
| 4 | 第 27 条：分配记录读者不经共用判定 | 同上：只有 `allocation_records_under_root: Ok(())` | 无 |
| 5 | 第 27 条：用户可见两棵树的读者不经共用判定 | 同上：只有 `user_visible_tree_root_pointers: Ok(())` | 无 |
| 6 | 第 27 条：挂载态把同一种树两条报成条目解不开 | 同上：只有 `open_pool_for_read` 报 `TreeTableEntryMalformed` | 无 |
| 7 | 第 36 条：所选根的 txg 改回裸加一 | txg 那条 → `recovery.rs:2161:56` 「attempt to add with overflow」（被测代码里的溢出检查先红，测试自己的断言走不到；工作区 `Cargo.toml` 的 `[profile.release]` 开着 `overflow-checks = true`，副本专用的 release 版同样红在这一行，`prove-red-logs-second/002.log`） | 无 |
| 8 | 第 36 条：加二才算溢出（边界） | txg 那条 → 第 398 行「最大值减一：加一不溢出，恢复读回第一个文件」（测试自己的断言） | 无 |
| 9 | 第 37 条：全池槽 0 都不可择时改回只试 4096 | `slot_one_is_found_…` → 第 499 行 expect「两块盘的槽 1 都在槽距 8192 处」（证红时那一版测试文件的行号，下同） | 另 5 条：512 那条、记的槽距不等那条、世代号那条、fsid 那条、上界那条 |
| 10 | 第 37 条：逐档按 1024 字节一档试 | 512 那条 → 第 525 行 expect「两块盘的槽 1 都在 4608 处」 | 无 |
| 11 | 第 37 条：逐档试不核记的槽距等于所在偏移 | 记的槽距不等那条 → 第 545 行 left `(8192, 9)` | 无 |
| 12 | 第 37 条：逐档试取第一个收到的、不比世代号 | 世代号那条 → 第 570 行 assert_eq（left `(4096, 1)`） | fsid 那条、上界那条 |
| 13 | 第 37 条：各盘择到的只比设备数、不比 fsid | fsid 那条 → 第 591 行 left `None` | 无 |
| 14 | 第 37 条：逐档试不含上界那一档 | 上界那条 → 第 617 行 assert_eq | 无 |
| 15 | 第 37 条：一槽可择的都没有时丢掉 incompat 那一槽 | 保留分流那条 → 第 645 行 let-else panic「槽 1 布局不认识：该报 incompat 位图不认识」 | 无 |
| 16 | 第 37 条：逐档试把 S 越界当成自证不过 | 保留分流那条 → 第 674 行 let-else panic「槽 1 的 S 越界：该整池拒」 | 无 |
| 17 | 环短于三条记录：mkfs 不按在飞上限拒 | 环那条 → 第 752 行 matches!「环 4096 字节装不下三条记录」 | 无 |
| 18 | 环短于三条记录：恰好三条也拒（边界） | 环那条 → 第 773 行「恰好三条记录长的环放行」 | 无 |
| 19 | 第 37 条：逐档试世代号相同时取偏移大的 | 世代号相同那条 → 第 592 行 left `(8192, 1)` right `(4096, 1)`（第二批，行号是那时的） | 无 |
| 20 | （表第 933 行改点名，见 `mutations-replacements.tsv`）槽 0 不可择的盘不借池里别的盘槽 0 记的槽距 | 借槽距那条 → 第 596 行 assert_eq「盘 0 按盘 1 槽 0 记的 4096 找到本池的槽 1，不去试 8192」（第三批，12 条那一版） | 无 |

- 日志：第一批 `/tmp/claude-1000/impl-rev-a2b/prove-red-logs/001.log`–`018.log`（依表中 1–18 的次序），第二批 `prove-red-logs-second/001.log`（第 19 行）、`002.log`（第 7 行的 release 版），第三批 `prove-red-logs-third/`。
- 行号取自证红时副本里那一版测试文件：第一批时文件里还没有后补的两条（定稿里它们插在「世代号」与「fsid」两条之间），所以 1–18 行里 fsid 那条及其后的行号比定稿的测试文件小；定稿里各条用例函数的起始行见「第 4 步」一节开头的 `grep -n`。
- 第三批另跑了一行副本专用的「表第 933 行照原样、只跑它点名的那一条」：`✗ … 没红 slot_one_is_looked_for_at_the_slot_spacing_another_devices_slot_zero_records 没判红`（`prove-red-logs-third/001.log`，`ok. 1 passed`）——这次改动之后那一行抓不到了，见「要主 agent 处理的」第 2 条。

## 要主 agent 处理的

1. **A2a 的测试按这次的定案必然红，文件不在我的单里。** `crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs` 的 `a_journal_ring_shorter_than_one_record_is_refused_before_any_write` 后半段（第 224–235 行）断言「一条记录长的环放行」；环短于三条记录 mkfs 拒之后它红在第 231 行（主工作区实跑，原样）：`一条记录长的环放行：Some(JournalRingHoldsFewerRecordsThanTheSafetyFactor { ring_bytes: 4096, minimum_ring_bytes: 12288 })`，`test result: FAILED. 13 passed; 1 failed`（日志 `/tmp/claude-1000/impl-rev-a2b/main-a2a-test.log`）。改法：第 226 行 `journal_ring_bytes: JOURNAL_RECORD_BYTES` 改成三条记录（`3 * JOURNAL_RECORD_BYTES`），第 233 行的消息与第 201 行文档里「恰好一条记录那么长放行」跟着改成三条。前半段（半条记录长的环报 `JournalRingShorterThanOneRecord`）不受影响：短于 4096 的环先撞上那一判。表里 A2a 那几行变异不用动：「短于一条记录的环不拒」那一行在我的改动下仍会让那条用例红（半条记录的环改报新成员，`matches!` 不中）——推的，没跑，因为那条用例此刻的基线是红的，`prove-red.sh` 在基线红时停。
2. **表第 933 行（「实审 A2a 第 37 条：槽 0 不可择的盘不借池里别的盘槽 0 记的槽距」）这次改动之后抓不到了。** 那一行把 `.or(slot_spacing_recorded_by_the_first_mountable_slot_zero)` 改成 `.or(None)`；改前这样一改，盘 0 就回落去按 4096 试、找不到槽 1，A2a 那条用例红；改后回落去逐档试，照样在 8192 找到，那条用例照旧绿（实跑：`副本专用：933 行照原样 … 没红`）。借槽距这一步现在只剩两个差别：少读 2032 档，以及盘上有别的槽距留下的旧槽时择到的不同。新用例 `a_device_whose_slot_zero_fails_borrows_the_spacing_another_slot_zero_records_instead_of_searching` 钉的就是后一个差别；把第 933 行改点名到它的整行替换放在 `/tmp/claude-1000/impl-rev-a2b/mutations-replacements.tsv`（名字、文件、原文、替换文不变，只换第五、六段），实跑抓到（`prove-red-logs-third/002.log`）。派发提示说不改 `crates/mutations.tsv`，这一行由主 agent 定换不换；不换，门禁 59 号会在它上面红。
3. **门禁 74 号在主工作区红，查过不是这次改动带来的。** 主工作区原样末行见下一节；在证红副本（开工快照 + 我这 4 份）里单跑同一个测试二进制，带我的改动与换回 `orig/` 里改动前的三份各跑一次：两次都是 `test result: FAILED. 21 passed; 3 failed`，红的是同样三条、panic 的行与消息相同，除计时与测试完成的先后之外输出逐行相同（`random-history-mine.log`、`random-history-orig.log`，比法：去掉计时行之后 `diff`，只剩 3 处先后）。主工作区只红其中两条（别的会话之后改好了一条）。
4. **`RecoveryFailure` 没有新开成员。** 两处新判（同一种树两条、txg 加一溢出）都报 `UnitMalformed`，`what` 各是一个新 `pub const`。`crates/singlefs-harness/src/history.rs` 的 `recovery_failure_member` 穷举这个枚举，加成员要同时改那份文件，它不在我的单里。要专门的成员（比如带上种类码的「树表里同一种树两条」），连 `history.rs` 一起改。
5. **「单元区起点随环长走」（调度记录里定的 ①，说并进 A2b）没做**：这次的规格与文件单里没有它。看到的现状：主工作区的 `allocator.rs` 正被别的会话改成按环长现算（`journal::slot_after_the_journal_ring`，还有一份 `core_review_unit_area_start_and_publish_limits` 测试在跑）。`recovery.rs` 里判分配记录落点的 `allocation_records_fit_the_pool_geometry`（第 1109、1114 行）仍按常量 `UNIT_AREA_START_SLOT`、并调 `unit_area_slots_of_device(device_bytes)`；那边若改了这个函数的签名或起点，这份文件要跟着改——它这一轮在我的单里，两边要排个先后。

## 停下交主 agent 的设计问题

没有停在半路的分支：四件都写到了底。下面几处是条款没逐字写、我按最近的已有条款取的，推翻就改。

1. **「多个候选取世代号最高、各盘 fsid 一致的」我读成逐盘取世代号最大的、再由原有那一判比各盘 fsid。** 依据是 D22（单元原子性怎么合成） 已定项 16「择槽取校验和过且世代号最大的（逐盘计）」加上原来槽 0、槽 1 那条路的做法。另一种读法是池级：先只留「各盘都有同一个 fsid 的候选」的那几份，再取世代号最大的。两者只在盘上有别的 fsid 的旧槽时分叉：盘 0 {旧池 fsid 世代号 5，本池世代号 1}、盘 1 {本池世代号 1} 时，我的读法整池报 `SystemConfigurationsDisagree`（用例 `devices_whose_highest_found_slots_disagree_on_the_filesystem_identifier_refuse_the_pool` 钉住），池级读法择本池、挂得上。我取的是拒的那一边。
2. **逐档试到的 incompat 位图不认识的槽不核偏移**（它的槽距字段在哪不认识的布局里说不准），一份可择的都没有时拿按偏移升序第一个这种槽报「布局不认识」；**S 越界的槽在任何一档上都整池拒**（与槽 0、槽 1 同一个判法，而它的槽距同样读不出来）。代价：盘上 4096 到 1 MiB 之间躺着一份别的布局或 S 越界的旧槽时，全池槽 0 都坏的那一格上报的是它，而不是「没有有效的系统配置」。两处都由 `the_search_keeps_the_incompat_and_slots_per_region_refusals_of_slot_one` 钉住（改前的 4096 那一档就是这么报的）。
3. **上界取 A2a 给的「根环基址 − 4096」**，没按今天 mkfs 的几何收紧：A2a 那条「固定结构两两不重叠」在 S ≥ 4 下把槽距压到 3 MiB ÷ 4 = 786432 以内，这 2033 档里最后 504 档（786944 到 1044480）今天的 mkfs 写不出来；留着是因为那条严的读法记着欠账（调度记录 ③），放宽到「只比同一块盘上的区域」时上界会跟着变，而「槽 1 在根环基址之前」不随它变。

## checker 要跟着改什么（归 B2，我没动 checker）

- 第 37 条同一个回落：`crates/singlefs-checker/src/image.rs` 此刻第 281 行 槽 0 解不出时按 `FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES` 读槽 1（A2a 报告里写的是第 261 行，B2 在改这份文件，行号漂了）。恢复现在在全池槽 0 都坏时逐档试；槽距大于 4096 的这种镜像上，恢复择得到系统配置、checker 读不到。
- 第 27 条：恢复与挂载态现在一律把「同一种树两条」判成损坏；checker 那边 `walk.rs` 此刻第 3497 行那一处把它判成「这条根算不算非空判不了」，有没有一条不变量直接判它，我没查全，交 B2。

## 层 0、崩溃注入、故障注入可能改判的（都是推的，没跑）

- 树表判重：写者每种树只写一条（`first_transaction_step_five_publish.rs` 等按种类数条目的用例都是七种各一条），层 0 与崩溃注入枚举出的每个崩溃状态上树表都是写者写的那一份 ⇒ 读者的结局不变、钉值不变。改判的只会是改出来的镜像。
- txg 加一溢出：只有 checkpoint_txg 是 u64 最大值的根走得到，写者写不出（txg 按次加一）⇒ 层 0、崩溃注入不变。
- 逐档试槽距：只在**全池**没有一个槽 0 可择时走。层 0 不生成撕裂态、mkfs 之后每块盘的槽 0 都在 ⇒ 层 0 不走它。走得到它的是改盘与故障注入：①「每块盘第 N 次读槽 0 报错」那一类（`second_transaction_step_three_formatted_pool.rs` 第 54 行起的 `fail_the_nth_read_of_system_configuration_slot_zero_on_each_device`，偏移恰好 0、逐盘数）在那一次择系统配置里两块盘的槽 0 都读不到 ⇒ 现在每块盘多读 2032 档（全落在偏移 0 之外，不改「第几次读槽 0」的数），槽距 4096 的池上择到的仍是 4096 那一份、世代号比较结果与改前相同；② 按「第 N 次调用」或「从第 N 次起每一次」摆注入点的随机故障注入（`fault_injection.rs` 的 `draw_faults`，`every_call_across_the_pool` 这类）若让两块盘的槽 0 同时读坏，多出来的 2032 × 盘数次读会把之后的注入点挪到别的读上；判的是「不 panic、没有已知红清单外的失败」，会不会翻只能跑了才知道。③ `system_configuration_per_device_redundancy.rs` 的「每块盘两槽都坏」与 `system_configuration_rollback_floor_and_layout_identity.rs` 的「四槽都是不认识的布局」「盘 0 坏、盘 1 布局不认识」那几条也走逐档试，按式子推结局与改前相同（一份候选都没有、报的成员与点名的盘不变），没跑。
- mkfs 新拒的环长：默认 768 MiB、`history.rs` 小盘 128 MiB、E158 那三档 3 MiB、E158 找小环从 48 KiB 起（`e158_root_choice_repair.rs` 的 `find_small_ring`）都 ≥ 12288 字节，过得了新判；按数值推的，没跑。仓里写死短于三条记录的环长的只有 A2a 那条用例（见「要主 agent 处理的」第 1 条）。
- 要在提交时跑的：层 0 全量、崩溃注入、故障注入快档、门禁 59 号（连这 19 行一起，第 933 行按上面第 2 条处置）。

## 第 4 步那几样的末尾原样输出

开跑前看负载：`ps -o pid,args -u "$(id -u)"` 没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`；别的会话的 `cargo test`（`checker_known_bad_images`、`crash_enumeration_sharded_across_processes`、`core_review_unit_area_start_and_publish_limits`、`rollback_floor_written_into_…` 等）在跑，照常 `nice -n 19` 跑，没等到锁的记录。全部命令经 `bash research/scripts/capped.sh 4`，跑编出来的代码的另经 `bash research/scripts/run-with-memory-cap.sh 8G`。

定稿的测试文件里各条用例的起始行（`grep -n '^fn a_\|^fn slot\|^fn the_\|^fn devices_'`，原样，含两个辅助函数）：

```
276:fn a_tree_table_carrying_one_kind_of_tree_twice_is_refused_as_damaged_by_every_reader() {
368:fn a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking(
414:fn slot_bytes() -> usize {
446:fn slot_at(image: &MemoryPool, device: DeviceIdentity, offset: u64) -> SystemConfiguration {
495:fn slot_one_is_found_by_trying_every_spacing_when_no_slot_zero_in_the_pool_verifies() {
520:fn a_slot_spacing_that_is_a_multiple_of_512_bytes_only_is_found_too() {
535:fn a_self_verifying_slot_whose_recorded_spacing_is_not_its_own_offset_is_not_taken() {
559:fn the_search_takes_the_highest_generation_even_when_it_is_a_stale_slot_of_an_earlier_make_filesystem(
583:fn a_device_whose_slot_zero_fails_borrows_the_spacing_another_slot_zero_records_instead_of_searching(
610:fn slots_of_equal_generation_found_by_the_search_resolve_to_the_smaller_offset() {
632:fn devices_whose_highest_found_slots_disagree_on_the_filesystem_identifier_refuse_the_pool() {
649:fn the_search_tries_the_largest_spacing_whose_slot_one_ends_at_the_root_ring_base() {
680:fn the_search_keeps_the_incompat_and_slots_per_region_refusals_of_slot_one() {
792:fn a_journal_ring_holding_fewer_records_than_the_safety_factor_is_refused_before_any_write() {
```

主工作区（我这 4 份与证红副本逐字节相同，`cmp` 核过），2026-09-27：

- 动到的测试二进制 `cargo test --offline -p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search`（`main-run-final.log`）：
  `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s`
- `cargo test --offline -p singlefs-core --lib`（`main-core-lib-final.log`）：
  `test result: ok. 127 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s`
- A2a 那个二进制（我的改动让它红，只为核「要主 agent 处理的」第 1 条；`main-a2a-test.log` 那一跑）：`test result: FAILED. 13 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s`
- `cargo build --offline --all-targets`（`main-build-final.log`）：退 0，`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.87s`
- `cargo fmt --check`：退 1，出 diff 的文件全是别的会话的（我这 4 份单独 `rustfmt --edition 2021 --check` 退 0）：
  ```
        5 crates/singlefs-core/src/admission.rs
        1 crates/singlefs-core/src/allocator.rs
       67 crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
        5 crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs
        2 crates/singlefs-harness/tests/core_review_unit_area_start_and_publish_limits.rs
  ```
- `check.sh` 那一套 lint 下的 `cargo clippy --offline --all-targets --all-features -- -D warnings -D clippy::wildcard_enum_match_arm …`（`main-clippy-final.log`）：退 101，报错的位置全在别的会话的文件里（`admission.rs` 3 处、`allocator.rs` 1 处：`manually reimplementing div_ceil`、`manual implementation of .is_multiple_of()`、`doc list item without indentation`），`singlefs-core` 在这几处停下，没轮到装置。末行原样：`error: could not compile \`singlefs-core\` (lib test) due to 4 previous errors`。
- 所以另在副本上核了我这几份的 clippy：`/tmp/claude-1000/impl-rev-a2b/clippy-copy`（证红副本的拷贝；开工快照里 B2 正在改的 `singlefs-checker/src/walk.rs` 有 4 处 clippy 红——`very complex type`、`needless_borrow`、两处 `shadow_unrelated`——这一份副本只在 checker 的 `lib.rs` 头上加了一行带 reason 的 `#![allow]` 放过它们）。同一套 lint 下 `cargo clippy -p singlefs-core --lib --tests --all-features` 退 0（`clippy-copy-core.log` 末行 `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.00s`：前一次整工作区那一跑里核心层已经过了），`touch` 过 `recovery.rs` 与新测试之后 `cargo clippy -p singlefs-harness --lib --test core_review_tree_table_duplicates_and_slot_one_search --all-features` 退 0（`clippy-copy-harness-mine.log` 末行 `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1.72s`，前两行是 `Checking singlefs-core`、`Checking singlefs-harness`）。整工作区 `--all-targets` 在那份副本上停在别人的测试 `checker_narrow_invariants_and_abandoned_roots.rs` 第 188、618 行的两处 `wildcard_enum_match_arm`。

登记给我的门禁阶段（主工作区，`nice -n 19 bash .claude/gate.d/<文件>`，74 号外面套 `capped.sh 4`；各自末行原样与退出码）：

| 阶段 | 退出码 | 末行 / 判定 |
|---|---|---|
| 33-mutation-tables.sh | 1 | `→ crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \n），改完单跑那几行证明点名的测试红。` 红的 6 行全在别的会话的文件里：第 430、436 行（`allocator.rs`）、第 974、975、977、978 行（`admission.rs`），「命中 0 次」；日志里提到我这 3 份文件的 0 处。我另核了表里文件是我这 3 份的 92 行，原文都恰好命中 1 次 |
| 53-format-const-placeholders.sh | 0 | `✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））` |
| 74-model-differential.sh | 1 | `签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。` 上方 `test result: FAILED. 22 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 46.02s`，红的是 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 与 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`；不是这次改动带来的，见「要主 agent 处理的」第 3 条 |
| 92-layout-checker-sync.sh | 0 | 末行 `      第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行）：.claude/kb/layout/02-second-txn.md`；成功句 `✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321，105 个格式常量里变了 7 个（checker 在同一次改动里跟了 7 个，按滞后表放行 0 个），都不欠 checker 跟进` |
| 94-checker-implementation-disjoint.sh | 0 | `    这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 \`crates/mutations.tsv\` 里，门禁 59 号复跑`；成功句 `✓ checker 与实现只共享常量模块 \`singlefs-format\`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 \`singlefs_core\`（别名引进来的 0 个）；共享模块 1 份源码的正文 283 行里没有分支与循环（\`#[cfg(test)]\` 标着的项 263 行不扫）` |
| 93-feature-bits.sh | 0 | `✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））` |
| 89-closeout-row27-preconditions.sh | 77（本次未跑） | `    「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐`；判定行 `⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）` |

各阶段完整输出在 `/tmp/claude-1000/impl-rev-a2b/gate-<阶段文件名>.log`。

## `git diff --stat -- crates litmus`（主工作区，2026-09-27，原样；别的会话同时在改 crates/，这张表分不出谁改的，我的文件以「这一轮写过的文件」一节为准；新测试文件还没被 git 跟踪，不在表里）

```
 crates/mutations.tsv                               |  678 +-
 crates/singlefs-checker/src/image.rs               |  216 +-
 crates/singlefs-checker/src/lib.rs                 |  106 +-
 crates/singlefs-checker/src/position_addressed.rs  |    2 +-
 crates/singlefs-checker/src/walk.rs                | 1915 +++--
 crates/singlefs-core/src/admission.rs              |  620 +-
 crates/singlefs-core/src/allocation_record_tree.rs |    2 +-
 crates/singlefs-core/src/allocator.rs              |  299 +-
 crates/singlefs-core/src/code_two_tree.rs          |    4 +-
 crates/singlefs-core/src/extent_tree.rs            |    2 +-
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/journal.rs                |   37 +-
 crates/singlefs-core/src/lib.rs                    |    2 +-
 crates/singlefs-core/src/make_filesystem.rs        |  199 +-
 crates/singlefs-core/src/mount.rs                  | 2557 +++++--
 crates/singlefs-core/src/mounted_read.rs           |   43 +-
 crates/singlefs-core/src/recovery.rs               |  712 +-
 crates/singlefs-core/src/rollback_witness.rs       |  314 -
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  277 +-
 crates/singlefs-core/src/transaction.rs            |  506 +-
 crates/singlefs-core/src/write_request_split.rs    |   51 +-
 crates/singlefs-format/src/lib.rs                  |   56 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        | 7368 ++++++++++++--------
 .../src/bin/e158_root_choice_repair.rs             | 6818 +++++++++++++++++-
 .../src/bin/first_transaction_on_device.rs         |  175 +-
 .../src/bin/first_transaction_region_bytes.rs      |    4 +-
 crates/singlefs-harness/src/crash.rs               | 2505 ++++++-
 crates/singlefs-harness/src/crash_injection.rs     |  821 ++-
 crates/singlefs-harness/src/device_log.rs          |   38 +-
 crates/singlefs-harness/src/fault_injection.rs     |  749 +-
 .../src/first_transaction_regions.rs               |   92 +-
 crates/singlefs-harness/src/history.rs             | 1122 ++-
 crates/singlefs-harness/src/lib.rs                 |   60 +-
 crates/singlefs-harness/src/model.rs               |  964 ++-
 crates/singlefs-harness/src/model_comparison.rs    |  338 +-
 crates/singlefs-harness/src/on_device_modes.rs     |    4 +-
 crates/singlefs-harness/src/read_tally.rs          |    2 +-
 crates/singlefs-harness/src/segments.rs            |  106 +-
 .../tests/checker_known_bad_images.rs              |  986 ++-
 crates/singlefs-harness/tests/common/mod.rs        |  108 +-
 .../tests/common_tree_split/mod.rs                 |  138 +-
 .../tests/first_transaction_region_bytes.rs        |   78 +-
 .../tests/first_transaction_step_five_publish.rs   |   22 +-
 .../tests/first_transaction_step_one_mkfs.rs       |    8 +-
 .../tests/first_transaction_step_seven_layer0.rs   |  233 +-
 .../singlefs-harness/tests/instance_acquisition.rs |   89 +-
 .../tests/parallel_line_one_sequential_write.rs    |    4 +-
 ...ansaction_parallel_line_one_last_record_flag.rs |    8 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |    4 +-
 ...ansaction_parallel_line_one_sequential_write.rs |    4 +-
 ..._transaction_parallel_line_three_many_inodes.rs |    2 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |   21 +-
 .../tests/second_transaction_step_five_reuse.rs    |  532 +-
 .../tests/second_transaction_step_four_rollback.rs | 2626 ++++---
 .../tests/second_transaction_step_one_overwrite.rs |    3 +-
 ...action_step_three_acquisition_barrier_layer0.rs |   29 +-
 ...second_transaction_step_three_formatted_pool.rs |  688 +-
 ...econd_transaction_step_three_second_instance.rs |    5 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  663 +-
 ...ond_transaction_step_zero_test_only_switches.rs |    2 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   24 +-
 ...transaction_supplement_three_crash_injection.rs |  658 +-
 ...transaction_supplement_three_fault_injection.rs |  412 +-
 ..._transaction_supplement_three_random_history.rs |  434 +-
 ...transaction_supplement_two_admission_formula.rs |  346 +-
 ...ent_two_c519_whole_device_loss_after_warm_up.rs |   29 +-
 ...two_c533_row_publish_record_without_its_root.rs |    7 +-
 ...ion_supplement_two_commit_generated_fallback.rs |   26 +-
 ...rop_and_devices_without_the_selected_version.rs |  258 +-
 ...nsaction_supplement_two_instance_table_chain.rs |  100 +-
 ...tion_supplement_two_instance_table_page_full.rs |   77 +-
 ...plement_two_instance_table_second_page_write.rs |    4 +-
 ..._two_multi_record_transaction_zero_publishes.rs |    5 +-
 ...action_supplement_two_presumed_clause_checks.rs |  105 +-
 ...plement_two_publish_failure_resent_unchanged.rs |    2 +-
 ...n_supplement_two_release_checksum_quarantine.rs |   88 +-
 ..._transaction_supplement_two_rollback_witness.rs | 1079 ---
 ...ction_supplement_two_rollback_witness_layer0.rs |  457 --
 ...n_supplement_two_root_ring_turn_in_one_mount.rs |    2 +-
 ...nt_two_row_publish_checks_before_acquisition.rs |    2 +-
 ..._two_tree_identifier_watermark_crash_orphans.rs |   72 +-
 ...ement_two_tree_nodes_and_the_central_mapping.rs |   10 +-
 ...second_transaction_supplement_two_tree_split.rs |  147 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |  314 +-
 .../system_configuration_mutability_classes.rs     |   35 +-
 88 files changed, 29885 insertions(+), 11189 deletions(-)
```

## 删掉的副本与留下的材料

- 删了（交回之前，`du -sh` 量的）：`/tmp/claude-1000/impl-rev-a2b/work-copy`（证红副本，开工快照 + 我这 4 份，自带 target，14G；它就是开工时建的 `/tmp/claude-1000/impl-rev-a2b/prefix-copy` 改名来的，那个路径已不在）、`/tmp/claude-1000/impl-rev-a2b/clippy-copy`（核 clippy 的那一份，368M）、`/tmp/claude-1000/impl-rev-a2b/mine-saved`（随机历史对比时暂存我那三份源文件，260K）。
- 留着（都在 `/tmp/claude-1000/impl-rev-a2b/`，没有仓副本、没有编译目录）：`mutations-append.tsv`（19 行）、`mutations-replacements.tsv`（1 行）、`orig/`（改动前那三份源文件）、`my-diff-*.patch`、`probe-version-of-the-test.rs`（改前跑的探针版）、`prefix-run.log`、`prove-red-logs*/`、`prove-red-run-*.log`、`random-history-{mine,orig}.log` 与跑它们的 `random-history-compare.sh`、`gate-*.log`、`main-*.log`、`copy-*.log`、`clippy-copy-*.log`、`git-diff-stat.txt`、`test-function-lines.txt`、`progress.md`。
- 这些都没入 `research/results/`：实现员的写范围不含 `research/`，由主 agent 定留不留、留哪份。

## 没做什么

- 没走三方对抗；没提交；没跑层 0（快档与全量）、崩溃注入、故障注入、QEMU、herd7、crates 变异整表（门禁 59 号）、全量 `cargo test`；这几样归提交时的 `crash-verifier` 与整轮门禁。
- 除了我自己的测试二进制与 `singlefs-core --lib`，只另跑了两个二进制：A2a 那一份（核它必然红的那一条）与门禁 74 号那一份随机历史（在副本里带 / 不带我的改动各一次，判 74 号的红是不是我带来的）。别的走得到这次改动的二进制（`system_configuration_per_device_redundancy`、`system_configuration_rollback_floor_and_layout_identity`、`second_transaction_step_three_formatted_pool`、`second_transaction_parallel_line_two_mounted_read`、`first_transaction_step_six_recovery` 等）没跑，结局是推的（「层 0、崩溃注入、故障注入可能改判的」一节）。
- 规格外、调度记录里说并进 A2b 的「单元区起点随环长走」没做（「要主 agent 处理的」第 5 条）。
- `RecoveryFailure` 没开新成员（第 4 条）；A2a 的测试文件没改（第 1 条）；`crates/mutations.tsv` 没改，第 933 行的替换交主 agent（第 2 条）。
- 没改 checker（「checker 要跟着改什么」一节交 B2）。
- 主工作区的 `cargo clippy` 与 `cargo fmt --check` 此刻红在别的会话的文件上，我没碰那些文件；我这几份的 clippy 只在副本上核过（见「第 4 步」一节）。
