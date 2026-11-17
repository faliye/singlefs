# 事务员出的事实表（线索，主 agent 只现查了正文点名的那几处）

### checker/src/image.rs（+52 / -10，11 块）

**1 这一次改了什么**
三块：L958-L975 journal_ring_bytes_lie_in_the_supported_range 的注释补「在飞上限装得进 4 字节一格（u32）」，并在判式里加一个条件 u32::try_from(in_flight_record_limit).is_ok()（L975）；L2314–L2440 七个测试去掉 a_ 前缀；L2462 起新增一个测试 journal_ring_whose_in_flight_record_limit_overflows_its_four_byte_field_lies_outside_the_supported_range（38 行），在 3<<44 与它减 12288 两个环长上钉住 u32 边界。

逐块：
- image.rs:955 +5/-2 〔fn physical_block_size_fits_a_root_slot(physical_b〕 加：/// journal 环长装得下 F 条记录（在飞记录数上限 = ⌊环长 ÷ 4096 ÷ F⌋ 不是 0，D23（journal 的角色与格式） 已定项 1 ｜ 删：/// journal 环长装得下 F 条记录（在飞记录数上限 = 环槽数 ÷ F 不是 0，D23（journal 的
- image.rs:972 +1/-0 〔fn journal_ring_bytes_lie_in_the_supported_range(〕 加：&& u32::try_from(in_flight_record_limit).is_ok()
- L2311–L2437：8 块纯换名（测试名去 a_/an_ 前缀或标识符换名，各 ±1~3 行），例：L2311 fn a_pool_holding_only_first_version_val→fn pool_holding_only_first_version_value；L2344 fn a_slot_declaring_seventeen_slots_per_→fn slot_declaring_seventeen_slots_per_re
- image.rs:2458 +38/-0 〔mod tests {〕 加：/// 在飞上限那 4 字节一格（`.claude/kb/layout/01-first-txn.md` 一「journal 在飞记录数上限 | 4」）：环长 

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- image.rs:958：D23（journal 的角色与格式） 已定项 18 ｜ /// journal 环长装得下 F 条记录（在飞记录数上限 = ⌊环长 ÷ 4096 ÷ F⌋ 不是 0，D23（journal 的角色与格式） 已定项 18；F 取格式常量
- image.rs:960：I-8.1 ｜ /// `.claude/kb/layout/01-first-txn.md` 一「journal 在飞记录数上限 | 4」；装不进时 I-8.1 那一格在飞上限的等式必不成立，
- image.rs:961：I-7.13 ｜ /// 这一格先拒，归 I-7.13、与实现收同一张表），且环的末端（同一槽「journal 环起点」字段 × 16384 + 环长）
去重：D23（journal 的角色与格式） 已定项 18、I-8.1、I-7.13

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 3 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 改到/新加的函数名 2 个（非测试函数）：同文件测试模块引到的 1 个，tests/ 目录文件引到的 0 个，两处都没引到（grep 名字）的 1 个：physical_block_size_fits_a_root_slot。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 1 行（键新出现）、消失 0 行、行变 14 行（多为必红测试名随改名变）。 新增行的变异名：checker 环长判据去掉「在飞上限装进 4 字节」一格（库单测）

**5 线索，没核**
- 没有。

### checker/src/lib.rs（+466 / -28，6 块）

**1 这一次改了什么**
三块：L3 头注释改写（去掉「步 1 能判的」）；L57–L200 在 crc32_castagnoli_table 加「按内容复用」：每条线程一份记录（thread_local，指纹=长度+16 个固定位置的字，候选逐字节比对，单线程上限 8 MiB，超过整份清空，一个指纹至多 4 段），新增类型 ContentFingerprint、RememberedChecksum、RememberedChecksumsOfThisThread 与 5 个常量/线程局部量，另有 cfg(test) 的计数；L1354–L1670 新增 315 行测试（on_a_fresh_thread 等辅助与 6 个 #[test]）；L1761–L1779 三个测试去 a_。

逐块：
- lib.rs:1 +3/-3 〔〕 加：//! 这一份判：系统配置槽（magic / 整槽校验和 / incompat 位 / 世代号）、根记录槽（magic / 整槽校验和 / fsid / fla ｜ 删：//! 步 1 能判的：系统配置槽（magic / 整槽校验和 / incompat 位 / 世代号）、根记录槽（mag
- lib.rs:54 +145/-21 〔pub fn crc32_castagnoli_bitwise(bytes: &[u8]) -> u〕 加：/// 每条线程记下的内容总字节数的上限（[`crc32_castagnoli_table`] 的按内容复用，每条线程各一份、互不相加）： ｜ 删：/// 与实现那份仍是不同写法（实现在 x86_64 上查到 SSE4.2 走硬件 `crc32` 指令，别处走 sli
- lib.rs:1351 +315/-1 〔mod tests {〕 加：/// 在一条新起的线程上跑 `body_on_the_fresh_thread`：按内容复用的记录与几个计数都是每条线程一份，新线程上从空的起， ｜ 删：fn a_system_configuration_slot_whose_node_size_is_not_the_fo
- L1758–L1776：3 块纯换名（测试名去 a_/an_ 前缀或标识符换名，各 ±1~3 行），例：L1758 fn a_first_incompat_byte_without_bit_two→fn first_incompat_byte_without_bit_two_i；L1767 fn a_first_incompat_byte_without_bit_one→fn first_incompat_byte_without_bit_one_i

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段新加 3 处；测试段 36 处 assert/expect/panic。生产段逐处：
  - lib.rs:154：.set(comparisons.get() + u64::try_from(compared).expect("候选数装得进 u64"));
  - lib.rs:163：let mut remainder = u32::try_from(index).expect("小于 256");
  - lib.rs:178：[usize::from(u8::try_from(remainder & 0xFF).expect("低 8 位") ^ *byte)]
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 11 个：const REMEMBERED_CONTENT_BYTES_LIMIT_PER_THREAD、const REMEMBERED_CANDIDATES_PER_FINGERPRINT_LIMIT、const FINGERPRINT_SAMPLED_WORD_COUNT、struct ContentFingerprint、struct RememberedChecksum、struct RememberedChecksumsOfThisThread、static REMEMBERED_CHECKSUMS_OF_THIS_THREAD、static REMEMBERED_CONTENT_BYTES_LIMIT_OVERRIDE_OF_THIS_THREAD、static FRESH_COMPUTATIONS_OF_THIS_THREAD、static SAME_FINGERPRINT_OTHER_CONTENT_COMPARISONS_OF_THIS_THREAD、const WORD_BYTES（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：关键词命中 生产段 2、测试段 1：
  - lib.rs:90：thread_local! {
  - lib.rs:100：thread_local! {
  - lib.rs:1359：std::thread::spawn(body_on_the_fresh_thread)
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 改到/新加的函数名 1 个（非测试函数）：同文件测试模块引到的 1 个，tests/ 目录文件引到的 1 个，两处都没引到（grep 名字）的 0 个。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 6 行（键新出现）、消失 0 行、行变 8 行（多为必红测试名随改名变）。 新增行的变异名：池级 checker 按内容复用：指纹相同的候选不做逐字节比对就取记下的值、池级 checker 按内容复用：复用那一支取同一指纹下最早记下那个候选的值、不取逐字节相同的那一个、池级 checker 按内容复用：记录到上限清空时只清总字节数、不清指纹表、池级 checker 按内容复用：现算之后不把内容与结果记下、池级 checker 按内容复用：同一指纹之下的候选数多留一个、不压在上限、池级 checker 按内容复用：checksum_field_holds 绕开按内容复用、直接按位算

**5 线索，没核**
- lib.rs:57-200 按内容复用的记录是 checksum 的进程内记忆，不是判定存放处，按「不新建判定库」格不算；但它是 thread_local 状态：同一线程内先后两次调用的结果相同只靠「候选与输入整段逐字节相同」保证，没有另核哈希碰撞外的路径
- lib.rs:60 注释点名 `.claude/heavy-thread-default`：8 MiB 乘线程数的估算依赖那份登记值，登记值变了这条注释不会跟着红，没核

### checker/src/position_addressed.rs（+2 / -2，2 块）

**1 这一次改了什么**
两个测试去 a_ 前缀（L399、L416），无其他改动。

逐块：
- position_addressed.rs:396 +1/-1 〔mod tests {〕 加：fn record_whose_last_slot_overflows_does_not_fit_in_any_leaf() { ｜ 删：fn a_record_whose_last_slot_overflows_does_not_fit_in_any_le
- position_addressed.rs:413 +1/-1 〔mod tests {〕 加：fn key_shorter_than_the_tree_key_width_names_no_child() { ｜ 删：fn a_key_shorter_than_the_tree_key_width_names_no_child() {

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 本文件没有改到非测试函数，或不是 Rust 源（coverage.txt 无条目）。
- crates/mutations.tsv：指到本文件的行，新增 0 行（键新出现）、消失 0 行、行变 2 行（多为必红测试名随改名变）。

**5 线索，没核**
- 没有。

### checker/src/rebuild.rs（+33 / -34，27 块）

**1 这一次改了什么**
三块：L132–L136、L157、L271、L474 把 ScanRebuildNotDecided 的三个成员由 …OnTheTotalOrderKeyOfI18 / …UnderTheTotalOrderKeyOfI18 一类改成 …MergedVersionTotalOrderKey / …UnderTheMergedVersionTotalOrder，调用点与 L1478、L1494、L1521 三处测试断言跟着换；其余 20 个测试去 a_ 前缀。

逐块（机器取：函数上下文、首个加行、首个删行；未逐块通读）：
- rebuild.rs:129 +3/-3 〔pub(crate) enum ScanRebuildNotDecided {〕 加：DataUnitVersionsTiedOnTheMergedVersionTotalOrderKey, ｜ 删：DataUnitVersionsTiedOnTheTotalOrderKeyOfI18,
- rebuild.rs:154 +3/-3 〔pub(crate) enum ScanRebuildNotDecided {〕 加：ScanRebuildNotDecided::DataUnitVersionsTiedOnTheMergedVersionTotalOrderKey => { ｜ 删：ScanRebuildNotDecided::DataUnitVersionsTiedOnTheTotalOrderKe
- rebuild.rs:268 +2/-2 〔fn latest_of_the_ordered_versions<Version: std::bo〕 加：ScanRebuildNotDecided::DataUnitVersionsTiedOnTheMergedVersionTotalOrderKey ｜ 删：ScanRebuildNotDecided::DataUnitVersionsTiedOnTheTotalOrderKe
- rebuild.rs:471 +1/-1 〔pub(crate) fn choose_current_versions(〕 加：return Err(ScanRebuildNotDecided::CopiesOfOneVersionCarryDifferentPayloadsUnderT ｜ 删：return Err(ScanRebuildNotDecided::CopiesOfOneVersionCarryDif
- rebuild.rs:1150 +1/-2 〔mod tests {〕 加：fn walk_import_behind_a_comment_naming_the_test_attribute_is_outside_the_use_lis ｜ 删：fn a_walk_import_behind_a_comment_naming_the_test_attribute_
- rebuild.rs:1169 +1/-1 〔mod tests {〕 加：fn walk_import_behind_a_comment_naming_the_test_attribute_is_named_outside_the_w ｜ 删：fn a_walk_import_behind_a_comment_naming_the_test_attribute_
- rebuild.rs:1209 +1/-1 〔mod tests {〕 加：fn second_line_that_is_exactly_the_test_attribute_is_refused() { ｜ 删：fn a_second_line_that_is_exactly_the_test_attribute_is_refus
- rebuild.rs:1475 +1/-1 〔mod tests {〕 加：Some(ScanRebuildNotDecided::DataUnitVersionsTiedOnTheMergedVersionTotalOrderKey) ｜ 删：Some(ScanRebuildNotDecided::DataUnitVersionsTiedOnTheTotalOr
- rebuild.rs:1491 +1/-1 〔mod tests {〕 加：Some(ScanRebuildNotDecided::CopiesOfOneVersionCarryDifferentPayloadsUnderTheMerg ｜ 删：Some(ScanRebuildNotDecided::CopiesOfOneVersionCarryDifferent
- rebuild.rs:1518 +1/-1 〔mod tests {〕 加：Some(ScanRebuildNotDecided::PackedRecordVersionsTiedOnTheMergedVersionTotalOrder ｜ 删：Some(ScanRebuildNotDecided::PackedRecordVersionsTiedOnTheTot
- L1562–L2128：17 块纯换名（测试名去 a_/an_ 前缀或标识符换名，各 ±1~3 行），例：L1562 fn a_whole_object_tombstone_kills_the_da→fn whole_object_tombstone_kills_the_data；L1589 fn a_tombstoned_object_that_still_has_it→fn tombstoned_object_that_still_has_its_

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- 加行的注释里没有 D、E、C、I 起头的编号或「已定项」「欠账」。

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 0 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 0 个。（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 改到/新加的函数名 2 个（非测试函数）：同文件测试模块引到的 1 个，tests/ 目录文件引到的 0 个，两处都没引到（grep 名字）的 1 个：latest_of_the_ordered_versions。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 1 行（键新出现）、消失 1 行、行变 17 行（多为必红测试名随改名变）。 新增行的变异名：扫描重建码 3 同键两版报成码 1 那一格（I-1.8 ②）

**5 线索，没核**
- rebuild.rs:132-136 错误成员改名（成员名里原有的编号 I18 去掉），属改名不属新成员，没核别的文件有没有旧名残留（本包 grep 未做）

### checker/src/walk.rs（+251 / -57，60 块）

**1 这一次改了什么**
主线一件事：给 I-3.5（版本引用没收全）加「树表里有走到没有条款的形状的条目」这一路。新增函数 tree_table_entry_is_a_shape_without_a_clause（L821）与 Walk::note_the_first_tree_table_entry_without_a_clause（L3748）；Walk、TreeTableEntryOfTheWalk、VersionPlacementsForTheAppearanceRule、VersionReferencesForTheAppearanceRule 各加一个字段（L718、L783、L843、L871）；judge_the_appearance_rule_of_adjacent_candidate_versions（L1090）里这样的版本令相邻一对报「不判」；begin 与 judge_the_tree_table_entries_an_earlier_version_walked 里置位/复位。其余：三个 fn 名去 a_（L15081 truncation_intent_names_the_record）、局部变量 a_/some_ 去前缀（L4333、L7856、L12840）、一处 /// 改 //（L7273）、约 40 个测试去 a_，最后 L28997 起新增 120 行 4 个测试钉住新路。

逐块（机器取：函数上下文、首个加行、首个删行；未逐块通读）：
- walk.rs:715 +2/-0 〔struct Walk<'reader> {〕 加：/// 这一版树表里第一条「走到没有条款的形状」的条目的种类（[`tree_table_entry_is_a_shape_without_a_clause`]）
- walk.rs:780 +2/-0 〔struct TreeTableEntryOfTheWalk {〕 加：/// 根指针整条全零：[`tree_table_entry_is_a_shape_without_a_clause`] 读它。
- walk.rs:806 +1/-0 〔fn tree_table_entry_of_the_walk(entry: &[u8]) -> O〕 加：root_pointer_is_all_zero: root_pointer.all_zero,
- walk.rs:814 +12/-0 〔fn tree_table_entry_of_the_walk(entry: &[u8]) -> O〕 加：/// 树表的一条条目是不是 I-3.5 条款说的「走到没有条款的形状」（`crates/singlefs-checker/invariants/05-空间记账
- walk.rs:840 +2/-0 〔struct VersionPlacementsForTheAppearanceRule {〕 加：/// 这一版树表里第一条「走到没有条款的形状」的条目的种类（[`tree_table_entry_is_a_shape_without_a_clause`]）
- walk.rs:868 +2/-0 〔struct VersionReferencesForTheAppearanceRule {〕 加：/// 这一版树表里第一条「走到没有条款的形状」的条目的种类：有就是引用集合数不全，它参与的每一对报不适用（[`VersionPlacementsForTheA
- walk.rs:1087 +13/-0 〔fn judge_the_appearance_rule_of_adjacent_candidate〕 加：if let Some(kind) = previous_versions
- walk.rs:1883 +1/-0 〔impl<'reader> Walk<'reader> {〕 加：kind_of_a_tree_table_entry_without_a_clause_in_this_version: None,
- walk.rs:2180 +1/-0 〔impl<'reader> Walk<'reader> {〕 加：self.kind_of_a_tree_table_entry_without_a_clause_in_this_version = None;
- walk.rs:2218 +2/-0 〔impl<'reader> Walk<'reader> {〕 加：kind_of_a_tree_table_entry_without_a_clause: self
- walk.rs:2267 +2/-0 〔impl<'reader> Walk<'reader> {〕 加：kind_of_a_tree_table_entry_without_a_clause: version
- walk.rs:2936 +1/-0 〔impl<'reader> Walk<'reader> {〕 加：self.note_the_first_tree_table_entry_without_a_clause(&entries_of_the_walk);
- walk.rs:3354 +2/-0 〔impl<'reader> Walk<'reader> {〕 加：// 没有 key 形态的条目走读走不下去：这一版的引用没收全由 `note_the_first_tree_table_entry_without_a_clau
- walk.rs:3743 +16/-0 〔impl<'reader> Walk<'reader> {〕 加：/// 这一版树表里第一条「走到没有条款的形状」的条目的种类记下（I-3.5）：先走这片树表的那一版与共享它的版本都经这里，
- walk.rs:3773 +1/-0 〔impl<'reader> Walk<'reader> {〕 加：self.note_the_first_tree_table_entry_without_a_clause(&entries);
- walk.rs:4330 +1/-1 〔impl<'reader> Walk<'reader> {〕 加：let some_copy_of_the_class_the_entry_type_names_verifies = ｜ 删：let a_copy_of_the_class_the_entry_type_names_verifies =
- walk.rs:4338 +1/-1 〔impl<'reader> Walk<'reader> {〕 加：if some_copy_of_the_class_the_entry_type_names_verifies { ｜ 删：if a_copy_of_the_class_the_entry_type_names_verifies {
- walk.rs:7270 +1/-1 〔fn extent_tree_with_the_inode_of_each_data_pointer〕 加：// 待读的一个节点：上段 (层级, 序号)，或某个 inode 下段 (层级, 序号)；根那一个层级取它自己头里的。 ｜ 删：/// 待读的一个节点：上段 (层级, 序号)，或某个 inode 下段 (层级, 序号)；根那一个层级取它自己头里的。
- walk.rs:7853 +2/-2 〔fn judge_release_generations(〕 加：let some_later_root_is_incomplete = scanned ｜ 删：let a_later_root_is_incomplete = scanned
- walk.rs:12837 +9/-9 〔fn judge_publish_ordinals_and_last_record_flags(〕 加：let some_record_sits_before_the_first_member = first_counter ｜ 删：let a_record_sits_before_the_first_member =
- L15078–L15208：3 块纯换名（测试名去 a_/an_ 前缀或标识符换名，各 ±1~3 行），例：L15078 fn a_truncation_intent_names_the_record(→fn truncation_intent_names_the_record(；L15134 a_truncation_intent_names_the_record(*in→truncation_intent_names_the_record(*inod
- walk.rs:23900 +1/-2 〔mod tests {〕 加：fn record_without_an_upper_leaf_entry_reddens_the_unit_count_only_when_its_size_ ｜ 删：fn a_record_without_an_upper_leaf_entry_reddens_the_unit_cou
- L24434–L27824：13 块纯换名（测试名去 a_/an_ 前缀或标识符换名，各 ±1~3 行），例：L24434 fn a_floor_raised_over_the_slot_of_the_g→fn floor_raised_over_the_slot_of_the_gen；L24453 fn a_floor_raised_over_the_slot_of_the_g→fn floor_raised_over_the_slot_of_the_gen
- walk.rs:28203 +1/-2 〔mod tests {〕 加：fn legal_deadlist_under_two_live_snapshots_holds_the_reference_intervals_and_the ｜ 删：fn a_legal_deadlist_under_two_live_snapshots_holds_the_refer
- walk.rs:28223 +1/-2 〔mod tests {〕 加：fn deadlist_entry_pinning_a_unit_the_head_still_references_violates_the_referenc ｜ 删：fn a_deadlist_entry_pinning_a_unit_the_head_still_references
- walk.rs:28239 +1/-2 〔mod tests {〕 加：fn deadlist_entry_pinning_a_unit_no_live_snapshot_references_violates_the_revers ｜ 删：fn a_deadlist_entry_pinning_a_unit_no_live_snapshot_referenc
- L28255–L28877：16 块纯换名（测试名去 a_/an_ 前缀或标识符换名，各 ±1~3 行），例：L28255 fn a_unit_only_a_live_snapshot_reference→fn unit_only_a_live_snapshot_references_；L28271 fn a_unit_only_a_live_snapshot_reference→fn unit_only_a_live_snapshot_references_
- walk.rs:28926 +1/-0 〔mod tests {〕 加：kind_of_a_tree_table_entry_without_a_clause: None,
- walk.rs:28936 +14/-1 〔mod tests {〕 加：reference_interval_verdict_of_the_appearance_rule_pair_given_the_tree_table_entr ｜ 删：let versions = [
- walk.rs:28963 +4/-0 〔mod tests {〕 加：versions[0].kind_of_a_tree_table_entry_without_a_clause =
- walk.rs:28994 +120/-0 〔mod tests {〕 加：/// 一个没登记的种类码：种类码登记表（D8（核心索引结构） 已定项 9）到 15 为止。

**2 改动处注释里引到的编号**（行号：原话；regex 取 I-x.y、D/E/C 数字、「已定项」、「欠账」）
- walk.rs:718：I-3.5 ｜ /// 这一版树表里第一条「走到没有条款的形状」的条目的种类（[`tree_table_entry_is_a_shape_without_a_clause`]）：这一版走完记进 I-3.5 的输入。
- walk.rs:817：I-3.5 ｜ /// 树表的一条条目是不是 I-3.5 条款说的「走到没有条款的形状」（`crates/singlefs-checker/invariants/05-空间记账.md` I-3.5 那一行）：
- walk.rs:817：I-3.5 ｜ /// 树表的一条条目是不是 I-3.5 条款说的「走到没有条款的形状」（`crates/singlefs-checker/invariants/05-空间记账.md` I-3.5 那一行）：
- walk.rs:3746：I-3.5 ｜ /// 这一版树表里第一条「走到没有条款的形状」的条目的种类记下（I-3.5）：先走这片树表的那一版与共享它的版本都经这里，
- walk.rs:3747：I-7.2 ｜ /// 共享的版本不重走条目，引用同样没收全。不进 `walk_failures`：I-7.2、I-4.8、I-7.4 判的是走读断没断，这一条由 I-9.16 ④ 判红。
- walk.rs:3747：I-4.8 ｜ /// 共享的版本不重走条目，引用同样没收全。不进 `walk_failures`：I-7.2、I-4.8、I-7.4 判的是走读断没断，这一条由 I-9.16 ④ 判红。
- walk.rs:3747：I-7.4 ｜ /// 共享的版本不重走条目，引用同样没收全。不进 `walk_failures`：I-7.2、I-4.8、I-7.4 判的是走读断没断，这一条由 I-9.16 ④ 判红。
- walk.rs:3747：I-9.16 ｜ /// 共享的版本不重走条目，引用同样没收全。不进 `walk_failures`：I-7.2、I-4.8、I-7.4 判的是走读断没断，这一条由 I-9.16 ④ 判红。
- walk.rs:28997：D8（核心索引结构） 已定项 9 ｜ /// 一个没登记的种类码：种类码登记表（D8（核心索引结构） 已定项 9）到 15 为止。
去重：I-3.5、I-7.2、I-4.8、I-7.4、I-9.16、D8（核心索引结构） 已定项 9

**3 检查单八格的原始事实**（只查加行；「生产段」指文件里 `mod tests` 之前，写在 `mod tests` 之外的 `#[cfg(test)]` 辅助也算在里面，要看行上方的 cfg）
- 走不到的理由：生产段没有新加 unreachable!/panic!/expect/unwrap/assert!；测试段 assert/expect/panic/unwrap 共 8 处。
- `_ =>`：生产段没有新加，测试段 0 处。
- `as` 转换：生产段没有新加，测试段 0 处。
- 类型与名字：生产段新加的条目名 3 个：fn tree_table_entry_is_a_shape_without_a_clause、fn note_the_first_tree_table_entry_without_a_clause、fn truncation_intent_names_the_record（错误成员/枚举成员的新增与改名见第 1 样逐块与概述）
- 不兼容旧代码与旧格式：加行没有命中（关键词 旧版|旧布局|旧格式|legacy|migrat|迁移|兼容旧|backward…）。
- 不新建判定库：加行没有命中（关键词 rocksdb|RocksDB|sqlite|prune.server|prun…）。
- 多机：本机独有状态：加行没有命中（关键词 hostname|/proc/|/sys/|env::var|std::env|…）。
- 多线程：加行没有命中（关键词 available_parallelism|--test-threads|ray…）。
- 带 --item 的命令与项 / 新崩溃流多罩了什么：加行（非注释）里提到 crash/崩溃的 0 处；本文件没有新加崩溃点重放流或崩溃用例。

**4 测它的测试与变异**
- 改到/新加的函数名 11 个（非测试函数）：同文件测试模块引到的 3 个，tests/ 目录文件引到的 2 个，两处都没引到（grep 名字）的 7 个：data_units_the_size_asks_for、extent_tree_with_the_inode_of_each_data_pointer_without_judging、judge_data_units_beyond_the_size_against_the_truncation_intents、judge_release_generations、note_the_first_tree_table_entry_without_a_clause、tree_table_entry_of_the_walk、truncation_intent_names_the_record。逐名明细在草稿目录 coverage.txt。
- crates/mutations.tsv：指到本文件的行，新增 13 行（键新出现）、消失 4 行、行变 152 行（多为必红测试名随改名变）。 新增行的变异名：实二十：I-8.9 不判一次发布的第一条不从 1 起、二·1 B7 守卫拿掉：比 L 晚的根走不完也照判、第30批 F-W：inode 树那一处另一份按类型段那一类自证过也不往下走（inode 叶容器预留段非 0、另一份自证成码 2，I-9.7 两种盘序都报不适用）、m4 截断截乙 size 之外有单元不要条目（I-4.10 T-2）、I-3.5（引用区间的精确性） 走到没有条款的形状：第一个走到树表的那一版不记（中间那一版种类没登记的镜像照判、红）、I-3.5（引用区间的精确性） 走到没有条款的形状：这一版走完不记进 I-3.5 的输入（中间那一版种类没登记的镜像照判、红）、I-3.5（引用区间的精确性） 走到没有条款的形状：共享树表的版本不照先走那一版的条目记、I-3.5（引用区间的精确性） 走到没有条款的形状：开始走下一版时不清上一版记的…另 5

**5 线索，没核**
- walk.rs:28997 常量 TREE_KIND_NOT_REGISTERED=16 的注释写「种类码登记表到 15 为止」，是测试里写死的 16；登记表再加一个种类，这条测试会误判成「没登记」，没核登记表此刻的最大码
- walk.rs:3747 注释写这一条「不进 walk_failures」「由 I-9.16 ④ 判红」：本表没去核 I-9.16 ④ 在 invariants 文件里的写法，没核

