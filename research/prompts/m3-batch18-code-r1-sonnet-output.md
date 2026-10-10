# 删除批第 18 批代码轮第一轮：云端正推腿（Sonnet）报告（2026-10-05）

本腿只判 K1（N1–N10 条款对实现）与 K3（19 份被判文件每份一格）。方向：从条款原文、设计轮判决格（`research/prompts/m3-delete-checker-r1-main-verification.md` 第二节）与用户定案（岔路单）推到冻结副本里的行，再推到钉它的测试与变异行；不攻（K2 归攻方腿）。代码行号都是冻结副本 `/tmp/claude-1000/m3-batch18-code-r1/tree/` 里的；除 `crates/mutations.tsv` 外，仓里主工作区同名文件今天与冻结副本逐字节相同（下面第 2 条命令），所以引文也能在仓里核。冻结副本里 `crates/mutations.tsv` 与仓里今天的不同（仓里已多了别批的行、行号错开），所以变异行一律写「冻结副本 mutations.tsv 第 N 行」，不写成文件:行号引文。

## 0 读过什么与复核的事实

读过：`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`「各条腿必须互不重复」、`.claude/rules/implementation-first.md`「规矩」、`.claude/kb/invariants.md` I-9.5 / I-9.6 / I-3.4 / I-4.4 / I-4.6 / I-4.7 六行与文件头、`.claude/kb/decisions/08-核心索引结构.md` 已定项 16、设计轮判决第二节与岔路单、`research/prompts/e172-preregistration.md` 1.2 / 5.3 / 5.6、`research/prompts/e142-r22-prereg.md` R67 与 5.4、`research/prompts/e161-r2-prereg.md` 5.6 / 九；冻结副本里 19 份被判文件的相关全文（walk.rs 8178–8620 与 9085–9310、image.rs 60–345、两个 wgsl 的删除态扫描段、五份 harness 测试、三个 bin、scenario.rs 的删除历史段）。没读禁读清单里的文件。

命令 1（冻结副本与快照 sha256 对表）：`cd /tmp/claude-1000/m3-batch18-code-r1/tree; sha256sum -c …/m3-batch18-code-r1-snapshot/crates-sha256.txt | grep -c ': OK$'` 输出 `19`；`grep -vc ': OK$'` 输出 `0`。
命令 2（仓里主工作区同表）：`sha256sum -c` 只有 `crates/mutations.tsv: FAILED`，其余 18 份 OK（仓里的 mutations.tsv 已有别批追加）。
命令 3（`IMPLEMENTED_INVARIANTS` 的条数与下标，脚本把常量名换成串）：输出 `declared 74 listed 74 distinct 74`，`{41: 'I-9.6', 65: 'I-3.5', 66: 'I-9.27', 67: 'I-5.5', 68: 'I-9.28', 69: 'I-9.5', 70: 'I-3.4', 71: 'I-4.7', 72: 'I-4.6', 73: 'I-4.4'}`。
命令 4（kb 状态列写「已实现」的 I-x.y 与代码清单对集合）：`grep -c '^| I-.*已实现' .claude/kb/invariants.md` 输出 `74`；脚本比两个集合，`in kb implemented not in code: []`、`in code not kb implemented: []`。
命令 5（变异表锚点）：冻结副本 mutations.tsv 的第 842 行（P1 小修改写）与第 2589–2627 行（x 第 2589–2597、y 第 2598–2601、P1 第 2602–2615、P1 小修第 2616、P2 第 2617–2627），逐行数「原文在目标文件里命中次数」：第一次脚本只数第 842 与 2602–2627 行，输出 `     27 1`（27 行各命中 1 次）；e161 的 16 行（第 1202–1213、2598–2601）另数一次，16 行各命中 1 次；每行第六段的测试名都在 `crates/` 某份 .rs 里有 `fn <名>`（脚本逐行 `True`）。第 2589–2597 行（x，三·6 与 E142 第 22 次的 9 行）另用同样的数法数过一次，9 行各命中 1 次。

实现今天的样子（主 agent 观测的前提之外，本腿自己读到的）：删除态扫描只有一处入口 `judge_deletion_state_of_the_versions`（`crates/singlefs-checker/src/walk.rs:9252` 调它），被判的版本清单在 9232 起拼；I-4.4 在 9259 起直接 `not_applicable`；I-9.6 墓碑那一半在 9290 起。没有 `crates/` 之外的实现可对。

## K1-N1　I-9.5 两个方向

条款（逐句）：
- kb：「队列 = inode 树里 nlink = 0 的号集合」（`.claude/kb/invariants.md:292`）；同一行写「① `deleted_inodes` 里的每个 inode 号在同一棵树的 inode 树里存在且记录的 nlink = 0」「② inode 树里 nlink = 0 的每条记录，号都在同一棵树的 `deleted_inodes` 里」。
- D8 已定项 16 第 3 条：「队列 = inode 树里 nlink = 0 的号集合」（`.claude/kb/decisions/08-核心索引结构.md:466`）。
- 设计轮判决：「I-9.5 能不能只红自己」一格定「采」：坏镜像取「队列里的号没有记录」（`research/prompts/m3-delete-checker-r1-main-verification.md:30`）。

代码：
- 方向 ①：「.find(|inode| records.get(inode).is_none_or(|record| record.nlink != 0));」（`crates/singlefs-checker/src/walk.rs:8419`）——没有记录、或记录 nlink ≠ 0 都红。
- 方向 ②：「.find(|(inode, record)| record.nlink == 0 && !queue.contains(inode))」（`crates/singlefs-checker/src/walk.rs:8439`）。
- 两个方向在函数「fn judge_the_deletion_queue_against_the_inode_records(」（`crates/singlefs-checker/src/walk.rs:8409`）里各 `judge` 一次，每一版每个方向各评一次；干净的一版也评过，所以「没有删除的版本上 I-9.5 报成立」成立。
- 队列根指针全零按空队列读（`deleted_inodes_queue_without_judging` 开头 `if root_pointer.all_zero`，`crates/singlefs-checker/src/walk.rs:8006`），与 D8 已定项 16 第 5 条「`deleted_inodes` 根指针置零」同向；inode 树根指针全零按零条记录读（8033）。
- 配对：「const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8 + 8;」（`crates/singlefs-checker/src/walk.rs:3429`）= 116，与 kb 写的「偏移 116 的头 ID」相同；`deleted_inodes` 条目的头 ID 对 inode 树条目偏移 0 的树 ID，配不上或配到不止一条走「Err(DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree)」（`crates/singlefs-checker/src/walk.rs:8347`）。
- 树表没有 `deleted_inodes` 条目的版本这一遍不判（循环只走 kind = 9 的条目）：不是漏，I-9.16 ④ 对非空树表要求各种类恰好一条（`judge_tree_table_entry_kind_counts` 的文档，`crates/singlefs-checker/src/walk.rs:3548`），缺一条由它红。

钉它的测试与变异行：
- 方向 ① 的「没有记录」形：测试 `a_queue_number_without_an_inode_record_reddens_only_the_deletion_queue_invariant`（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:61`），断言 `violated_invariants` 恰好等于 I-9.5 一条；冻结副本 mutations.tsv 第 2602 行（`first_queued_inode_without_a_record_of_nlink_zero.is_none(),` 换 `true,`）。
- 方向 ② 两形：僵尸记录（同文件第 85 行）与形式化线 being-deleted 形（同文件第 109 行）；变异第 2603、2604 行；第 2615 行把 nlink 偏移读成 32，由第 85 行那条红。
- **没钉的一格**：方向 ① 里「记录在、nlink ≠ 0」那一支（`|| record.nlink != 0`）没有坏镜像也没有变异。`invariants.md:292` 自己写了「还差「队列里的号在树里 nlink ≠ 0」那一形的坏镜像」，主 agent 背景也写已进第 19 批；本腿只记下：把 `is_none_or(|record| record.nlink != 0)` 改成 `is_none()` 今天没有任何一条测试会红（我读了 `pool_checker_judges…` 全部 12 条与 `checker_known_bad_images.rs` 的登记，没有一条造 nlink ≠ 0 的队列号）。

判定：N1 与条款、判决、D8 一致。推翻它的现象：造一份「队列 {1}、1 号记录 nlink = 1」的镜像，I-9.5 不红；或造一版 inode 树有 nlink = 0 记录而 `deleted_inodes` 条目存在、队列不含它，I-9.5 不红。

## K1-N2　I-4.7 对象取丙、I-4.6 乙、I-3.4 只落标签 4 恒 0

条款：
- I-4.7：「extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 的记录的号」（`.claude/kb/invariants.md:183`）加「inode 树里 nlink = 0 的记录」两样，要么在队列里要么单元全放；判决 F1 定丙（`research/prompts/m3-delete-checker-r1-main-verification.md:21`）。
- I-4.6：「size 为 0 时 0 或 1 个单元，size > 0 时 ⌈size ÷ 数据单元净荷容量⌉」（`.claude/kb/invariants.md:182`）；用户 2026-10-05 弹窗选乙（岔路单 `research/prompts/m3-delete-checker-r1-forks.md:10` 那一行状态列「用户已定（2026-10-05 选乙）」）。核心侧同式：「content_length_in_bytes.div_ceil(payload_capacity).max(1)」（`crates/singlefs-core/src/write_request_split.rs:61`）。
- I-3.4：「记账标签 4（待删除但意图未完成占用）恒 0」（`.claude/kb/invariants.md:148`），同一行「缺行判红」；判决 F5-a 只落这一句、F5-b 判红、F5-c 逐版（`research/prompts/m3-delete-checker-r1-main-verification.md:27`）。

代码逐支：
- I-4.7 对象集：nlink = 0 的记录的号并上「**data_units > 0 && !records.get(inode).is_some_and(|record| record.nlink >= 1)」（`crates/singlefs-checker/src/walk.rs:8472`），即「有单元且没有 nlink ≥ 1 的记录」（没有记录、或记录 nlink = 0）；判法「queue.contains(&inode) || data_units_left == 0,」（`crates/singlefs-checker/src/walk.rs:8480`）。与条款两样对象、一条判据逐句相同。
- 单元数怎么数：`extent_tree_with_the_inode_of_each_data_pointer_without_judging`（`crates/singlefs-checker/src/walk.rs:4114`）读 extent 树时每个数据指针带上它属于的 inode 号（上段叶内联的归条目 key 里的号，下段叶的每条记录归引它那条上段条目的号），数的时候滤掉全零数据指针（`data_unit_count_of_each_inode_without_judging`）；旧的 `extent_tree_without_judging`（`crates/singlefs-checker/src/walk.rs:4087`）改成包它，行为不变（其余调用方 `collect_tree_references` 只取指针）。
- I-4.6：「let data_units_the_size_asks_for = size_in_bytes.div_ceil(payload_capacity_in_bytes);」（`crates/singlefs-checker/src/walk.rs:8506`），size 为 0 走「data_units <= 1」（`crates/singlefs-checker/src/walk.rs:8508`），否则「data_units == data_units_the_size_asks_for」（`crates/singlefs-checker/src/walk.rs:8510`）；只对 nlink = 0 的记录判（射程取乙1）。净荷容量 checker 自己算一份 `DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET`（`crates/singlefs-checker/src/position_addressed.rs:26`），与核心侧 32634 同数（wgsl 里的 `DATA_UNIT_PAYLOAD_CAPACITY` 也是 32634，`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:107`）。
- 与 D8 已定项 16 第 2 条「size 改成「剩下单元数 × 净荷容量」」（`.claude/kb/decisions/08-核心索引结构.md:465`）对：意图那一版 size 不变（2.5 个净荷、3 个单元，⌈2.5⌉ = 3），每批后 size = 剩下单元数 × 净荷（⌈⌉ = 剩下单元数），最后一批删记录——没有「size > 0 而单元数不等」的合法中间态，也没有 nlink = 0、size = 0 而单元数 > 1 的合法态。
- I-3.4：「judgements.judge(invariant, has_the_pending_deletion_row, || {」（`crates/singlefs-checker/src/walk.rs:8290`）缺行红；「judgements.judge(invariant, value == 0, || {」（`crates/singlefs-checker/src/walk.rs:8297`）每一行标签 4 的值 0；每棵种类 4 的记账树走全了才判，根指针全零的树跳过、读不全的树报射程里判不了（`collect_accounting_rows_without_judging` 的 Err）。判法按版：`judge_pending_deletion_occupancy_of_a_version`（`crates/singlefs-checker/src/walk.rs:8259`）在 `judge_deletion_state_of_the_versions`（`crates/singlefs-checker/src/walk.rs:8178`）里每个被判的版本各调一次。
- 一处措辞差：kb 写「那一行」（单数），代码判的是「设备段 0xFFFF_FFFF 的标签 4 行至少有一条」，且对所有标签 4 行（含别的设备段）都比值为 0，不比行里的树 ID 与代段（代段另由 I-3.13 比）。第一版只有一个头一棵记账树，没有可分的合法状态。

钉它的测试与变异行：
- I-4.7：孤儿单元 `data_units_of_an_inode_without_a_record_outside_the_queue_redden_only_the_inode_being_deleted_invariant`（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:133`），变异第 2605 行（对象漏了孤儿形）与 P1 小修的第 2616 行（`true ||` 使 I-4.7 恒成立，由 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs:172` 那条红）。
- I-4.6：`an_inode_being_deleted_whose_size_asks_for_more_data_units_than_it_has_reddens_only_the_intermediate_state_invariant`（同文件第 158 行），变异第 2606（不比）、2607（⌈⌉ 改向下取整，由 `every_published_step_of_unlinking_a_three_unit_file_holds_the_deletion_invariants` 第 342 行那条红，因为意图那一版 size 是 2.5 个净荷）。
- I-3.4：值非 0 第 182 行、缺行第 201 行；变异第 2608、2609；中间版写坏最新根写对的用例第 240 行（I-3.4）、第 274 行（I-4.6），变异第 2611、2612（`candidate_indexes` 换成只有最新根）。
- **没钉的几格**：（一）I-4.6 的 size = 0 一支（`data_units <= 1`）：今天唯一 nlink = 0、size = 0 的记录是僵尸 inode 2（0 个单元），把 `<= 1` 改成 `== 0` 或 `< 1` 没有任何用例会红；（二）I-4.7 与 I-4.6 在中间版的「中间版写坏最新根写对」只钉了 I-3.4 与 I-4.6，没有 I-4.7、I-9.5 的中间版用例；（三）I-3.4 的「别的设备段上的标签 4 行值非 0」。这三格都不是条款与代码冲突，是钉的覆盖面。

判定：N2 与条款、判决、用户定案一致（I-4.6 只判单元数对 size；I-3.4 只落标签 4 恒 0；缺行判红；逐版）。推翻它的现象：一个合法已发布状态上 I-4.6 或 I-4.7 红（误红），或一份「size = 0、nlink = 0、2 个单元」的镜像 I-4.6 不红（漏判）。

## K1-N3　判哪几版、怎么配、读不全怎么报

- 「判哪几版」与 I-5.5 是不是同一份：删除态扫描的清单是候选集每一条根加由记录施加出来的几版——「let versions_to_scan_the_deletion_state_of: Vec<VersionToScanTheDeletionStateOf<'_>> =」（`crates/singlefs-checker/src/walk.rs:9232`）起，根取「tree_table_pointer: &root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],」（`crates/singlefs-checker/src/walk.rs:9239`），记录施加出来的取 `version.tree_table_and_mapping_root_pointers().0`；I-5.5 那一份是「tree_table_pointer: &root.record_bytes[36..122],」（`crates/singlefs-checker/src/walk.rs:9091`）加同一个 `.chain(versions_applied_only_by_records…)`（第 9094 与 9242 行各一处），而 `ROOT_RECORD_TREE_TABLE_POINTER` 是 36..122（`crates/singlefs-checker/src/walk.rs:7104`）：两份遍历的是同一个 `candidate_indexes` 与同一个 `versions_applied_only_by_records`，同一份清单。GPU 内核同形（`judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark` 与 `judge_deletion_state_of_the_versions` 各自两个 for，候选根 + `c_version_count`）。
- 单元相同只扫一次：「if !tree_tables_scanned.insert((」（`crates/singlefs-checker/src/walk.rs:8192`），键是树表指针第一份位置的 (设备, 槽, 校验和)，与 kb「树表单元相同的几版内容相同，只扫一次」一致。
- 配对：见 N1；extent 树按头 ID = 这棵 inode 树的树 ID 配（`kind_of(entry) == TREE_KIND_EXTENT && head_identifier_of(entry) == inode_tree`，`judge_deletion_queue_of_a_version`，`crates/singlefs-checker/src/walk.rs:8311`），配不上或不止一条报 `ExtentTreeOfTheInodeTreeNotSingle`。kb 的共用判法只写了 `deleted_inodes` 与 inode 树的配对，extent 树怎么配由 I-4.7 行「判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法」带过；代码把它读成同一种配法，没有越出。
- 三种情形各报什么：
  - 读不全 / 配不上：`Judgements` 记进 `part_of_the_range_not_judged`，`into_report` 里排在违例与部分没判之后、成立之前：「} else if let Some(reason) = self.part_of_the_range_not_judged.get(invariant) {」（`crates/singlefs-checker/src/image.rs:311`）报 `NotApplicable(reason)`——有违例红，没违例整条不适用带理由、不报成立。kb：「树表、`deleted_inodes` 或 inode 树读不全的版报 `part_of_the_range_not_judged`」（`.claude/kb/invariants.md:292`），文件头四种判定里「部分没判」今天只有 I-9.28 用（`.claude/kb/invariants.md:6`），C701 那一问没答之前这几条用「不适用带理由」，一致。
  - 根指针全零：树表指针全零的版「if tree_table_pointer.all_zero {」（`crates/singlefs-checker/src/walk.rs:8188`）跳过；树表 0 条「Some(tree_table) if tree_table.entries.is_empty() => None,」（`crates/singlefs-checker/src/walk.rs:8201`）不判不记缺口；队列根全零 = 空队列、inode 根全零 = 零条记录、extent 根全零 = 零单元（`data_unit_count_of_each_inode_without_judging` 的 `root_pointer.all_zero`，`crates/singlefs-checker/src/walk.rs:8113`）、记账树根全零跳过（8273）。这与 D8 已定项 16 第 5 条「`deleted_inodes` 根指针置零」同向。
  - 没有删除：I-9.5 因两个方向都评过而成立；I-4.7 报「走读到的每一版里都没有处于删除中的 inode（nlink = 0 的记录、有数据单元而没有 nlink ≥ 1 记录的号）」（`crates/singlefs-checker/src/walk.rs:8249`），I-4.6 报「走读到的每一版 inode 树里都没有 nlink = 0 的记录」（`crates/singlefs-checker/src/walk.rs:8253`）；全版没有 `deleted_inodes` 条目时 I-9.5 报「走读到的每一版树表里都没有 deleted_inodes 条目（树表指针全零或 0 条）：没有队列可比」（`crates/singlefs-checker/src/walk.rs:8241`）。三条理由都非空；与 kb「没有删除的版本上 I-9.5 报成立……I-4.7 与 I-4.6 没有对象时报不适用并带理由」逐句同。
- 钉住的与没钉住的：九个 `DeletionScanGap` 变体（`crates/singlefs-checker/src/walk.rs:7907`）里，只有 `InodeTreeUnreadable` 有删除线自己的用例（第 311 行那条，变异第 2613 行）和 `AccountingTreeUnreadable` 由「条目宽不是字段表宽」那条旧用例顺带钉（`["I-3.1", "I-3.11", "I-5.2", "I-9.6", "I-3.13", "I-3.4"],` 在 `crates/singlefs-harness/tests/checker_known_bad_images.rs:3489`，没有对应变异）；其余七个——`TreeTableUnreadable`、`TreeTableEntryShorterThanTheFieldTable`、`DeletedInodesHeadIdentifierNamesNoSingleInodeTree`、`DeletedInodesRootUnreadable`、`ExtentTreeOfTheInodeTreeNotSingle`、`ExtentTreeUnreadable`、`TombstoneTreeUnreadable`——在 `crates/` 里没有用例也没有变异（我 grep 了它们的变体名与理由串：变体名只在 walk.rs 出现；`checker_known_bad_images.rs` 第 8429 行有一句「extent 树读不全」，是 I-5.5 自己的理由，不是这个变体的）。树表去重、版本清单里「记录施加出来的几版」那一半、「记账树 / 墓碑树读不全」同理没有删除线的用例。

判定：N3 与 F4-a/b/c、F5-c 与 kb 共用判法一致；版本清单与 I-5.5 同一份；三种情形的报法与 kb 一致。缺口是钉得少（七个读不全变体零用例），不是写得与条款不同。推翻它的现象：造一份读不全的中间版，五条里有一条红而不是不适用；或造一份清单里「由记录施加出来」那一版写坏的镜像，删除线五条不红。

## K1-N4　下标次序与 I-4.4 理由句

- 下标：「pub const IMPLEMENTED_INVARIANTS: [&str; 74] = [」（`crates/singlefs-checker/src/image.rs:88`），末尾五项 I-9.5、I-3.4、I-4.7、I-4.6、I-4.4（`crates/singlefs-checker/src/image.rs:158`起，最后一项在第 162 行「INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,」）。命令 3 的输出已证下标 69–73 依次是 I-9.5、I-3.4、I-4.7、I-4.6、I-4.4，74 条互不重复，前 69 条的次序没动（69 之前最后一项仍是下标 68 的 I-9.28）。GPU 覆盖表同长：0..=73（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:74`），wgsl 的下标常量 69–73（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:152` 起）。
- I-4.4 恒报不适用：入口里直接 `not_applicable`，理由取常量（`crates/singlefs-checker/src/walk.rs:9261`），不看镜像，所以「没有删除」「删除中」两类镜像都报同一句；用户定案：岔路单「并写理由（第一版的意图载体没有可装增量的字段）」（`research/prompts/m3-delete-checker-r1-forks.md:8`）。
- 理由句与 kb 那一行：kb 写「理由逐字是 crates/singlefs-checker/src/image.rs 的 INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION」（`.claude/kb/invariants.md:179`），即 kb 把常量当逐字的权威。常量是「第一版不适用：意图载体（nlink = 0 的记录、`deleted_inodes` 的 key、墓碑）没有可装增量的字段；logged_ops 有字段表时实现」（`crates/singlefs-checker/src/image.rs:237`），kb 同一行的叙述是「第一版不适用：意图载体（nlink = 0 的记录、`deleted_inodes` 的 key、墓碑）没有可装增量的字段，checker 报不适用并写理由；logged_ops 有字段表时实现」（`.claude/kb/invariants.md:179`）——常量比 kb 叙述少「，checker 报不适用并写理由」这半句，别的逐字相同；常量上方的注释写「照 .claude/kb/invariants.md I-4.4 那一行的状态列」（`crates/singlefs-checker/src/image.rs:236`）。两边谁是权威 kb 已写明（常量），所以不算冲突；字面上两句不是逐字相等，记一笔。
- 钉：干净镜像 `"I-4.4" => InvariantVerdict::NotApplicable(INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION),`（`crates/singlefs-harness/tests/checker_known_bad_images.rs:1843`），每个删除步骤上同（`every_published_step_of_unlinking_a_three_unit_file_holds_the_deletion_invariants`，`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:342`），变异第 2614 行去掉 `not_applicable`。`INVARIANTS_WITHOUT_A_BAD_IMAGE` 点名它（`crates/singlefs-harness/tests/checker_known_bad_images.rs:2924`）。
- 一处过期注释：image.rs 里「GPU 判器内核还没判它们」（`crates/singlefs-checker/src/image.rs:86`）是 P1 写的，P2 合入后内核已判下标 69–72，注释过期（不影响行为；第 77、83 行同形的两句是更早批次留下的，也已过期）。

判定：N4 一致，下标次序、I-4.4 恒不适用与用户定案对得上；理由句与 kb 叙述差半句、以常量为准。推翻它的现象：`IMPLEMENTED_INVARIANTS` 里 I-4.4 不在下标 73，或 I-4.4 在某个镜像上报了不适用以外的判定。

## K1-N5　I-9.6 墓碑那一半

- 条款：「水位 > 所选根（最新的那条根）墓碑树里已发布的（按所选根指着的实例表判）墓碑容器里全部记录的对象 ID 最大值」（`.claude/kb/invariants.md:293`）；被抛弃时间线的墓碑不算。
- 「已发布」用的谓词：`PublishedPredicate`（挂载根的 (实例代号, txg) 加最新根指着的那一版实例表，「let published = PublishedPredicate {」`crates/singlefs-checker/src/walk.rs:9167`，与 I-1.8 扫描方向同一份对象），每个墓碑容器用它的头里诞生代号与写序实例代号判：「let is_published = published.holds_for(」（`crates/singlefs-checker/src/walk.rs:8592`）、不已发布的「if !is_published {」（`crates/singlefs-checker/src/walk.rs:8598`）整片容器跳过；谓词的两支码 2 / 码 3 同取法、只看诞生代号与实例代号（`WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit`，`crates/singlefs-checker/src/walk.rs:6028`）。读的树是所选根的树表：「&roots[newest_index].2.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],」（`crates/singlefs-checker/src/walk.rs:9293`）。两层都挡「被抛弃时间线」：只从所选根往下走（抛弃线上写的容器本来就不被所选根引用），再按谓词滤一遍。
- 判法：「judgements.judge("I-9.6", watermark > largest_tombstone_object, || {」（`crates/singlefs-checker/src/walk.rs:9298`）；没有已发布墓碑「Ok(None) => {}」（`crates/singlefs-checker/src/walk.rs:9304`）不判；墓碑树读不全「Err(gap) => judgements.part_of_the_range_not_judged("I-9.6", gap.reason()),」（`crates/singlefs-checker/src/walk.rs:9305`）。三种情形与 kb「一条已发布的墓碑都没有时没有对象，墓碑树读不全报射程里判不了」一致。取全部记录（容器级滤，不看记录自己的死亡 txg），与 kb「墓碑容器里全部记录的对象 ID 最大值」一致。
- 钉：测试 `a_tombstone_whose_object_identifier_reaches_the_inode_number_watermark_reddens_only_the_watermark_invariant`（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:222`，只红 I-9.6），变异第 2610 行；意图之后 / 每批之后 / 最后一批之后 I-9.6 成立（第 342 行那条）。
- **没钉的**：「被抛弃时间线的墓碑不算」这一句本身——`if !is_published { continue; }` 这一支没有 CPU 侧的用例或变异（把 `continue` 删掉没有测试红）；GPU 侧 `w_published` 那一支（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2508`）同样没有。墓碑树多层（类型段 0）一支、读不全一支也没有用例。

判定：N5 与 I-9.6 那一行的两半一致，用的是 I-1.8 / I-1.2 同一个已发布谓词；钉得少（谓词那一支零用例）。推翻它的现象：造一份所选根的墓碑树里有一个「写序实例代号的实例表行把它判为未发布」的容器，对象 ID ≥ 水位，I-9.6 仍红（应不红）；或所选根墓碑树里已发布的墓碑对象 ID ≥ 水位而 I-9.6 不红。

## K1-N6　坏镜像与用例

- 八份坏镜像各自一条测试，断言都是 `assert_eq!(violated_invariants(&image), vec![目标])`（恰好等于，不是 contains），并先 `assert_the_base_image_is_clean`（干净那一版一条违例都没有）：I-9.5 ①（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:61`）、I-9.5 ② 两形（85、109）、I-4.7（133）、I-4.6（158）、I-3.4 两形（182、201）、I-9.6（222）。`violated_invariants` 只收 `Violated`（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs:182`）：别的条被带成不适用不算「红」，这与验收「每条只红自己」的字面一致，但这几份镜像把别的条带成不适用与否没有被断言。
- 登记：八份镜像在 `common_deletion_in_progress` 里按基镜像分成四个 `known_bad_images_*`（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs:521` 起），`checker_known_bad_images.rs` 只把它们的目标名并进 `targets` 与 `IMPLEMENTED_INVARIANTS` 比集合（「.chain(common_deletion_in_progress::known_bad_images_after_the_intent_of_a_three_unit_file().iter())」，`crates/singlefs-harness/tests/checker_known_bad_images.rs:3014`）；该测试末尾「for (invariant, mutation) in cases {」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:3032`）的 `cases` 来自 `known_bad_images(&clean)`，**不含**这八份：登记只保证每条有名字，每份镜像「只红自己」由各自的专用测试断言，两者靠同一个函数名接上，没有一处断言登记里的目标名等于专用测试里的目标名。
- 中间版写坏、最新根写对：I-3.4（240）与 I-4.6（274），先断言中间版的单元槽号与最新根的不同（只被中间版引用），再改，断言只红目标且说明点名中间版的 txg；变异第 2611、2612 行把清单换成只有最新根。I-9.5、I-4.7 没有这种用例。
- 读不全报不适用：第 311 行那条把中间版的 inode 叶两份都盖坏（不补校验和），断言 I-9.5、I-4.7、I-4.6 三条都是 `NotApplicable` 且理由含「inode 树读不全」（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:332`）；变异第 2613 行。
- 干净镜像与不适用表：`checker_known_bad_images.rs` 干净镜像上 I-4.7、I-4.6 的不适用理由串与 walk.rs 的串逐字相同（`NO_INODE_BEING_DELETED_IN_ANY_WALKED_VERSION` 对 `crates/singlefs-checker/src/walk.rs:8249`）；`writable_mount_of_a_formatted_pool.rs` 五张表分别加 5、5、3、5、3 条（见 K3），长度 31 → 36、35 → 40、6 → 9、24 → 29、7 → 10，数与「74 − 成立数」对：74 − 36 = 38、74 − 40 = 34（文档里的「38 条」「34 条」）。
- 「变异改坏的是不是名字说的那一处」：我逐行对了第 2602–2615 行的名字与旧串新串。第 2602 行名字「队列里的号不核记录与 nlink」，替换把整个方向 ① 判据换成 `true`（同时去掉没有记录与 nlink ≠ 0 两个条件，测试只钉前一个）；第 2603 与 2604 行是同一处替换（`first_record_of_nlink_zero_outside_the_queue.is_none(),` 换 `true,`）挂两个必须红的测试；第 2611、2612 行只改根那一半清单，记录施加出来的几版仍在。其余名字与替换一致。

判定：N6 一致（「恰好」是 `assert_eq!` 恰好，不是 `contains`）。需要主 agent 知道的：（一）方向 ① nlink ≠ 0 一形无镜像（kb 已写）；（二）登记与专用测试只靠名字对接；（三）别的条被带成不适用没断言。推翻它的现象：把八份镜像之一换一种造法，目标之外的条跟着红。

## K1-N7　P1 连带红的一条用例与一处 lint

- 用例：`two_inodes_whose_extents_name_one_data_unit_in_one_root_redden_the_disjoint_ranges_and_the_deletion_accounting_invariants`（`crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs:172`），断言改成「["I-5.1", "I-4.7"],」（`crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs:225`）：次序是 `IMPLEMENTED_INVARIANTS` 的次序（I-5.1 下标 20、I-4.7 下标 71）；另断言 I-4.7 的说明点名「inode {下一个号} 处于删除中（inode 树里没有它的记录）」，I-5.1 的说明仍含「被同一版引用了两次」。
- 与判决 F1（取丙）的关系：这份镜像把 1 号的内联数据指针抄给 2 号，2 号在 inode 树里没有记录、extent 树里有数据单元、号不在队列里，正好落在 I-4.7 对象第二样（「extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 的记录的号」，`.claude/kb/invariants.md:183`）：I-4.7 红是条款的后果，不是误红；判决表 F1 那一行写的「只红自己」坏镜像是「孤儿单元」那一形（`research/prompts/m3-delete-checker-r1-main-verification.md:21`），这份镜像本来就同时违反 I-5.1，改成两条都红对得上。
- 变异表：第 842 行把必须红的测试名换成新名、第 2616 行新增 I-4.7 一判恒成立（`true ||`），旧名在 `crates/`、kb 里已无引用（只剩 `research/scripts/memory-peaks.tsv` 第 8367 行一条本机内存峰值记录，该文件不进 git，过期的只是峰值键，不影响判定）。
- lint：`unit` → `account_unit` 在 `remount` 里四处同时改（`crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs:119` 起 119、131、132、159），没有行为变化。

判定：N7 一致。推翻它的现象：这份坏镜像上 I-4.7 不红（那说明对象第二样没落实），或除这两条外有第三条红。

## K1-N8　GPU 判器：内核的每一支对 CPU 判法

读法：把 wgsl 的删除态扫描段（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2060` 起到 2536）逐函数对 walk.rs 8178–8620、9232–9310。「同」= 条件与次序相同；GPU 在 CPU 记成「射程里判不了」的地方 `return` 不置位（段首注释自己写明，`crash_judge_checker_scan.wgsl:2061`）。

| 支 | CPU | GPU | 对 |
|---|---|---|---|
| 版本清单 | 候选根 + 记录施加出来的版（`walk.rs:9232` 起） | `judge_deletion_state_of_the_versions`：候选根树表指针在根记录偏移 36，记录版在 `JOURNAL_NEW_ROOT_SEGMENT_OFFSET`（`crash_judge_checker_scan.wgsl:2428`），与同文件 I-5.5 那一份（`:2049`）两个 for 相同 | 同 |
| 树表门 | 全零跳过；0 条不判；任一条目窄于 200 记缺口（`walk.rs:8188`、8201） | 见下方引文 (a)，直接 return | 同（缺口不置位） |
| 去重 | 树表单元相同只扫一次（8192） | 不去重 | 违例位是并集，等价 |
| I-3.4 | 缺行红（8290）、每个标签 4 行值 ≠ 0 红（8297）；叶条目宽 ≥ 34、内部条目宽 ≥ 108、孩子层级 = 父 − 1、孩子指针非零，否则缺口 | 见下方引文 (b)，读树的形状检查逐条同（`:2238` 起，108 在 `:2080`） | 同 |
| 配头 ID | inode 树条目恰好一条、extent 树条目恰好一条，否则缺口 | 「if (inode_tree_entry_count != 1u) {」（`crash_judge_checker_scan.wgsl:2317`）、「if (extent_tree_entry_count != 1u) {」（`crash_judge_checker_scan.wgsl:2349`）都在判过 I-9.5 之后/之前 return，次序同（extent 配不上时 I-9.5 仍判） | 同 |
| 读队列 | 根全零空队列；层级 0、key 宽 8、条目宽 ≥ 8 | 同（`queue_present` 与三个条件，`crash_judge_checker_scan.wgsl:2322`） | 同 |
| 读 inode 树 | 同号后读盖先读；类型段 2/0/其它；记录宽 ≥ 140；层级差一；子指针非零 | `d_read_inode_records` 倒着压栈保持条目序、`d_note_inode_record` 同号覆盖、同五项检查 | 同 |
| I-9.5 ① ② | `walk.rs:8419`、8439 | 见下方引文 (c)、(d) | 同 |
| 数单元 | 滤掉全零数据指针，上段内联属条目号、下段属引它的上段号 | `r_count_data_unit_of_inode` 挂在 `r_extent_tree` 两处（`:312`、`:324`），全零指针不数；`r_noting`/`r_collecting…` 由每个调用方自己设，不漏到别的阶段 | 同 |
| I-4.7 | 8472、8480 | 「if (sw(count_row + 2u) > 0u && !has_a_record_of_nlink_one_or_more && !d_queue_contains(queue, queue_present, inode)) {」（`crash_judge_checker_scan.wgsl:2372`）；只遍历有单元的号，nlink = 0 记录零单元的号自己就成立，与 CPU 等价 | 同 |
| I-4.6 | 8506–8510 | 「var matches_the_size = data_units <= 1u;」（`crash_judge_checker_scan.wgsl:2384`）后 size ≠ 0 时用 `divmod64` 向上取整比等 | 同 |
| I-9.6 墓碑 | 8533–8620、9290 | 水位行在、读最新根墓碑树、码 2 层级差一且 ≥ 1、类型段 0/1/其它、记录宽 ≥ 40、「if (!w_published(container.birth_txg, container.instance, false, vec2<u32>(0u, 0u))) {」（`crash_judge_checker_scan.wgsl:2508`）、「if (has_a_published_tombstone && !lt64(largest_object_identifier, watermark)) {」（`crash_judge_checker_scan.wgsl:2523`）；树表门是「entry_count > 0 且宽 < 200」才 return（`:2454`），与 CPU 的「任一条目窄」及空表放行同 | 同 |
| 表容量 | 无上限 | inode 记录 1024、有单元的号 1024（`crash_judge_checker_common.wgsl:764`、771），满了置 `UNDECIDABLE_TOO_MANY_ENTRIES` 交 CPU | 不会出分歧，只会交回 |

表里三格的原句（表格里写不了竖线，挪到这里）：
- (a) 树表门：「if (!tt.ok || tt.entry_count == 0u || tt.entry_width < DELETION_TREE_TABLE_ENTRY_BYTES) {」（`crash_judge_checker_scan.wgsl:2406`）。
- (b) I-3.4 判红：「if (!has_the_pending_deletion_row || some_pending_deletion_value_is_not_zero) {」（`crash_judge_checker_scan.wgsl:2291`）。
- (c) I-9.5 ①：「if (row == NONE || sw(row + 2u) != 0u) {」（`crash_judge_checker_scan.wgsl:2337`）。
- (d) I-9.5 ②：「if (sw(row + 2u) == 0u && !d_queue_contains(」（`crash_judge_checker_scan.wgsl:2345`）。

读下来没有一支与 CPU 不同。数值：草稿区 = 290 544 + 1024 × 5 + 1024 × 3 = 298 736，wgsl 的 `SCRATCH_END`（「const SCRATCH_END: u32 = 298736u;」，`crash_judge_checker_common.wgsl:776`）与 Rust 的 `SCRATCH_WORDS`（「pub const SCRATCH_WORDS: usize = 298_736;」，`crash_judge_gpu.rs:119`）相等，Rust 里有测试把两者对起来（`crash_judge_gpu.rs:286` 的 `declaration`）；每条草稿通道 `floor(2_147_483_644 ÷ (298 736 × 4))` = 1797、12 GiB 装 10 782 个（我用 `python3 -c` 算过：`1194944 1797 10782`），与测试里的「assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 1797);」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:882`）、「assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 10_782);」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:889`）相同。

八条新对拍（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`）：
- 只比违例位的六条（`compare_the_checker_on_a_deletion_flow`，`:4619`；恢复结局在删除中间态上两边本来对不上，C712，所以不比）：删除进行中整条流（`:4653`，要求 I-9.5/I-3.4/I-4.7/I-4.6 在流上成立过且一个状态都不红）、队列带无记录的号（`:4680`，I-9.5）、标签 4 行值 1（`:4695`）、标签 4 缺行（`:4710`）、nlink = 0 记录的 size 多一个净荷（`:4754`，I-4.6）、墓碑对象 ID 抬过水位（`:4770`，I-9.6）。它们共用一条流 `record_new_pool_through_the_first_deletion_batch`（`:4383`）：新池新建文件、写三单元、意图、第一批。
- 连恢复一起比的两条（`assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening`）：孤儿单元（`:4726`，I-4.7）、僵尸记录（`:4741`，I-9.5），流是「新池新建文件 + 一个空 inode」，第一个文件仍可读。
- **覆盖到的格**：I-9.5 ① 的无记录形、② 的僵尸形；I-3.4 值与缺行；I-4.7 孤儿；I-4.6 size 过大；I-9.6 墓碑过水位；删除进行中（意图、第一批）整流零红。
- **漏的格**（GPU 与 CPU 在这些格上的一致没被对拍钉住，读代码判相同）：I-9.5 ① 的 nlink ≠ 0 一支；I-4.6 的 size = 0 一支；最后一批之后（记录删净、队列根全零、墓碑仍在）这一版；读不全的版（CPU 报不适用、GPU 必须不置位）——八条里没有一条造坏树；已发布谓词判未发布的墓碑容器；多层 inode 树 / 记账树 / 墓碑树；挂载续做在会话里 / H2 那类流；表满交 CPU 的路径。

判定：N8 读代码一支不差；对拍钉得到的格和漏的格如上。推翻它的现象：在 harness 档里用 CPU 判法算出期望、造一份上面「漏的格」里的镜像，CPU 与 GPU 违例位不同（GPU 不跑，所以我只能给读代码的结论）。

## K1-N9　E142 导出 `--device-bytes` 与 `run_deletion_history`

- 解析：「[] => Ok(TEST_IMAGE_DEFAULT_BYTES),」（`crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:96`）；「[option, value] if option == DEVICE_BYTES_OPTION => value.parse::<u64>().map_err(|error| {」（`crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:97`）；别的写法（缺值、多给、别的选项名、数解不开）走兜底臂退 1。每盘字节数用在建盘处（`:144`）与 `impl_config` 的 `image_bytes=`（`:178`）；其余参数（`e142_parameters(512, 512)`）不随盘大小变，与 R67「其余照主点」同（`research/prompts/e142-r22-prereg.md` 读法 R67 那一行）。不带参数时输出与旧 bin 逐字节相同的理由：旧串 `image_bytes={TEST_IMAGE_DEFAULT_BYTES}` 与新串 `image_bytes={device_bytes}` 在零参数时同值（diff 里只改了这一处与建盘处），只有 stderr 的拒绝句子变了，stdout 不受影响。三条内联测试 + 变异第 2593–2595 行（缺省改成 8 GiB、按 u32 解、什么选项都收）。
- 登记 5.3 H1/H2 对照 `run_deletion_history`（`crates/singlefs-harness/src/scenario.rs:548`）：E142 参数两块 4 GiB 内存盘、`physical_block_size` = io_min = 512（bin 常量与 `TEST_IMAGE_DEFAULT_BYTES` = 4 294 967 296，`crates/singlefs-format/src/lib.rs:308`）→ mkfs → 取号 → 暖机 → 新池新建文件（`run_new_pool_file_creation_keeping_the_allocator`，`scenario.rs:158`，`run_new_pool_file_creation` 改成包它，`scenario.rs:153`，行为不变）→ 同实例顺序写 3 个数据单元（内容 2.5 个净荷，`SECONDS_FROM_THE_FIRST_FILE_TO_THE_SEQUENTIAL_WRITE_IN_THE_DELETION_HISTORIES` = 60，`scenario.rs:233`）→ 取 `operations_before_the_unlink`（窗口起点）→ 意图（调用方时间 = 那条记录现有的写入时间，`scenario.rs:583`；登记 H3 才改成 +60）→ H1「every_batch_in_process」批大小 1（`deletion_history_batch_size`，`scenario.rs:248`）至多 `intent.data_pointers.len()` = 3 批到队列空（`scenario.rs:631`）；H2 意图后 `unmount`（`ShadowLedger::On`）→ 同一组盘 `mount_writable_with_test_only_switches_and_deletion_switches`（装批大小 1 的开关）→ 续做到空，挂载那一截按 core 的发布写账切段（`scenario.rs:687`）。逐步与登记 1.2「H1 意图与每一批在同一个进程里发到队列空；H2 意图之后正常卸载→同一组盘重开可写挂载→挂载续做走到空」一致。H2 若挂载推了抬 F、清扫、续做没走空会报错而不是静默切错（`publishes_this_history_does_not_attribute`）。
- 导出 bin：不收参数（`crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs:224`），两段历史按 `EVERY_HISTORY` 顺序打（`:228`），窗口起点取 `run.operations_before_the_unlink`（`:149`），符合登记 5.6 第 1 步「核 S1：有 deletion_write_dump.rs、scenario.rs 里有 run_deletion_history」。
- 钉：`deletion_scenario_writes.rs` 四条用例——两条切段单测（`spans_of_consecutive_parts_…`）、两条历史用例（进程里意图 13、批 12 / 11 / 10 单元每盘；挂载续做意图 13、批 12 / 13 / 12），变异第 2589–2592 行（scenario.rs）与 2596–2597 行（deletion_write_dump.rs）。
- 没在 K1 判的：非 4 GiB 时「每个区域是不是都随盘大小算」归 K2。

判定：N9 与登记 5.3 / 5.6 与 E142 第 22 次读法 R67 一致。推翻它的现象：零参数时输出与旧 bin 的主产物不逐字节相同；或 H1/H2 的某一步与登记 1.2 的顺序不同（例如 H2 的暖机在续做之后）。

## K1-N10　E161 装置对登记 5.6 I-1 到 I-12

逐条对 `research/prompts/e161-r2-prereg.md` 5.6 表（该表在第 220 行起）；e161 bin 简称「装置」，行号都在 `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`。

- I-1 第一条流段长：「const FIRST_STREAM_SEGMENT_LENGTHS: [usize; 11] = [2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2];」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:572`），41 次写在 573、根 3 在 575。一致。
- I-2 第二条流：78 段表（第 576 行起，我用脚本数：`78 481`，长度 78、和 481）、「const SECOND_STREAM_WRITES: usize = 481;」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:581`）、根 19（582）、取样段 11–26（584、585）、共享前缀 11（586）；字段改名：「name=stop_clause_shared_prefix bases_identical={} shared_prefix_segments={SHARED_PREFIX_SEGMENTS} shared_prefix_segments_identical={} clause_S2_holds={}」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:4324`）。第 16 项是 18 写段（表里下标 16），与 I-6 的 PC3 段对。一致。
- I-3 甲：「const STRIDED_SEGMENT_WRITES: usize = 24;」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:2771`），判法仍是 `<` 与 `==`（取样域里 `segment.len() == STRIDED_SEGMENT_WRITES` 在 `strided_segment_heads`）。一致。
- I-4 锚点：「const ANCHOR_FIRST_STREAM_SEGMENT_STATES: [u64; 11] = [8, 3, 1, 8, 3, 1, 8, 16_777_215, 3, 1, 8];」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:3923`），第二条流 16 项取样段（3928–3930）、总数 17_170_476（3931）、按发布「const ANCHOR_SECOND_STREAM_BY_PUBLISH: [u64; 4] = [16_777_219, 262_163, 65_547, 65_547];」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:3933`）、D1-small 45 / 393_261、stride 7_260、quick 46 / 52、段头「const ANCHOR_SEGMENT_HEADS_PER_STREAM: u64 = 4_096;」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:3941`）；`anchor_stream` 行带 `d1_heads=` 与 `d1_heads_match=`（4113）。我自己按段模型（每段 3^m·2^(n−m)−1，m 的认法照登记）手算了第一条流 11 项与第二条流 16 项的逐段数与两个总数（16_777_260、17_170_476），都对得上。一致。
- I-5 整条流：`SECOND_STREAM_FULL_STATES` 1_678_377_204（102）、`SECOND_STREAM_LARGEST_SEGMENT_STATES` 268_435_455（104）、恒等式「2u128.pow(28) == SECOND_STREAM_LARGEST_SEGMENT_STATES + 1」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:4149`）、`whole_stream_states_match=`、`whole_stream_largest_segment_match=` 进 `anchor_stream` 行（4104）。一致。
- I-6 PC3：「const TIMING_CONTROL_SEGMENT: usize = 16;」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:5029`）；开头核写数 18、状态数 ≥ 1000、序号都在那一段（`TimingControlSegmentShape::holds`，4940），不成立六个区一律不过并报 `pc3_segment_holds=false`（4965）；判决行字段名由 `format!("second_segment_{TIMING_CONTROL_SEGMENT}_pc3")` 现拼（5658），值 16 时就是 `second_segment_16_pc3`。一致。
- I-7 域守卫：锚点报完之后、评任何状态之前「if !evaluated_domain_sizes_hold {」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:5478`）打判决行并 `return 3`（5485）；守卫只看 D1-small、D1-quick、段头（4140）；测试共用函数评之前先「"域守卫：第一条流 D1-small 是 45 个状态（重跑登记 A4），对不上就不评"」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:5715`）。一致。
- I-8 A2 测试：断言次序 A1 装置对 crates、A1 crates 对公式、A2、A5 掩码宽 49，另加「formula_segment_state_count(24, 0),」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:5784`），测试名没改（5748）。一致。
- I-9 A4 测试 `stride_and_quick_domains_have_the_registered_sizes`（5791）：45 / 7_260 / 46 / 4_096；`evaluate_the_first_stream_small_domain`（5698）里 45。一致。
- I-10 新测试 `second_stream_shape_and_sampled_domains_have_the_registered_sizes`（5831）：78 段、481、19 根、两条流前 11 段相同、取样段逐段数、393_261 / 7_260 / 52 / 4_096、PC3 段 18 写 262_143 状态、整条流 1_678_377_204 与最大一段 268_435_455；不评任何状态。一致。
- I-11 config：「domain=first_small_45+quick_46_52+largest_segment_heads_4096」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:5426`），同一行有 `rerun_registration=research/prompts/e161-r2-prereg.md`。一致。
- I-12 变异表：冻结副本 mutations.tsv 里 e161 的 16 行（第 1202–1213 行旧 12 行与第 2598–2601 行新 4 行）原文在源码里各命中 1 次；M13–M16 的改法与登记第九节逐条对（24 → 26、`TIMING_CONTROL_SEGMENT` 16 → 14、第二条流第一取样段 11 → 10、`3u64.pow` → `2u64.pow`），必须红的测试名也对。内联测试 14 条（`grep -c '#\[test\]'` 输出 `14`，登记 5.8 C2 要现数）。一致。

判定：N10 十二条逐条一致。这一步没有跑装置、没有跑内联测试（checker 档，归主 agent）。推翻它的现象：主 agent 跑 C2 时有内联测试红，或 M13–M16 之一施加后必须红的测试没红。

## K3　19 份被判文件，每份一格

快照名单 `research/prompts/m3-batch18-code-r1-snapshot/crates-sha256.txt` 19 行（命令 1 已证冻结副本逐份 OK）。每份写：判到什么、依据在哪一节、什么推翻它。判定词只用三个：一致 / 冲突 / 规则没说；括号里写「有未钉的格」是指条款与代码一致、但没有测试或变异钉住的分支。

- F1 `crates/mutations.tsv`（冻结副本）：一致（有未钉的格）。第 842 行改写 + 第 2589–2627 行 39 行追加；28 行锚点命中数、9 行 x 的锚点命中数、16 行 e161 锚点命中数都是各 1 次（命令 5）；必须红的测试名都存在。要说的两处：第 2603 与 2604 行是同一处替换挂两个测试；第 2602 行把 ① 的两个条件一起换成 `true`，测试只钉其一。推翻：门禁 crates-mutation-replay 里有一行没红或锚点腐化；我没跑变异（归主 agent 提交时）。
- F2 `crates/singlefs-checker/src/image.rs`：一致。74 条、下标 69–73、五个新常量与理由句（N4）；一处过期注释（第 86–87 行）、理由常量与 kb 叙述差半句（kb 以常量为准）。推翻：下标或常量与 kb 不同。
- F3 `crates/singlefs-checker/src/walk.rs`：一致（有未钉的格）。删除态扫描一整组（N1 / N2 / N3 / N5）；未钉的格：方向 ① 的 nlink ≠ 0、I-4.6 的 size = 0、九个缺口变体里的七个、树表去重、已发布谓词 false 的一支。规则没说的一处：extent 树怎么配头 ID（kb 共用判法只写了 `deleted_inodes` 与 inode 树，代码把 extent 读成同一种配法）。推翻：N1–N5 各节的推翻现象。
- F4 `crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs`：一致。H1、H2 两段、窗口起点、不收参数（`deletion_write_dump.rs:224`），对登记 5.3 / 5.6 第 1 步；两条内联测试、两行变异（2596、2597）。推翻：主 agent 按 5.6 第 2 步跑出的导出里段数 / 步号与登记 1.2 不同（我没跑）。
- F5 `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs`：一致。零参数缺省 4 GiB、`--device-bytes <十进制>`、别的写法退 1（N9）；其余参数不随盘大小变与 R67 同。推翻：零参数输出与旧 bin 的主产物不逐字节相同（执行员报告已写第二段 matched=29 equal=29，那是实验侧读数，我没复核）。
- F6 `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`：一致。I-1 到 I-12 逐条对上（N10）。推翻：C2 内联测试有一条红，或 M13–M16 之一没红。
- F7 `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`：一致。五个下标常量 69–73（`crash_judge_checker_common.wgsl:152`–160）、两张删除态表 `D_INODE_RECORDS` 290544 起 1024 × 5、`D_DATA_UNIT_COUNTS` 295664 起 1024 × 3、`SCRATCH_END` 298736（算术：290544 + 5120 = 295664，+ 3072 = 298736）。推翻：`SCRATCH_END` 与 Rust 的 `SCRATCH_WORDS` 不等（Rust 侧有测试，`crash_judge_gpu.rs:283` 一带）。
- F8 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`：一致（有未钉的格）。删除态扫描与 I-9.6 墓碑那一半逐支对 CPU（N8 的表），读不全不置位，表满交 CPU；`r_extent_tree` 里新加的计数钩子在 `r_counting_data_units_of_each_inode` 关着时直接 return（`:82`），不影响别的阶段。未钉：读不全 / 谓词 false / 多层树 / 表满。推翻：任一格上 GPU 与 CPU 违例位不同（需显卡，我没跑）。
- F9 `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs`：一致。只改测试常数（1847 → 1797、11 086 → 10 782），我用 `python3 -c` 算出 `1194944 1797 10782`，与新常数相同。推翻：`SCRATCH_WORDS` 再变而这里没跟着变。
- F10 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs`：一致。覆盖表 0..=73 共 74 项（编译期断言要求第 k 项是 k）、`SCRATCH_WORDS` 298 736、文档段落。推翻：覆盖表漏一项（编译不过）。
- F11 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`：一致（有未钉的格）。八条新对拍，六条只比违例位、两条连恢复；覆盖与漏的格见 N8。我数了 `grep -c '^#\[test\]'` 全文件 65 条，新增的 8 条在 4653–4770 行。推翻：主 agent 提交时 `--features verdict-store,gpu` 跑出任一条红；我没跑。
- F12 `crates/singlefs-harness/src/scenario.rs`：一致。`run_deletion_history` 的 H1、H2 与登记 5.3 逐步对（N9）；`run_new_pool_file_creation` 改成包 `run_new_pool_file_creation_keeping_the_allocator`，行为不变。推翻：`deletion_scenario_writes` 四条里有一条红。
- F13 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs`：一致。只把 `unit` 改名 `account_unit`（四处），无行为变化（N7）。推翻：该文件的测试结果变了。
- F14 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`：一致。断言改成 I-5.1 与 I-4.7 都红，I-4.7 红是丙对象的后果（N7）；变异第 842、2616 行跟着。推翻：该坏镜像上 I-4.7 不红或第三条红。
- F15 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：一致。干净镜像上 I-4.7 / I-4.6 不适用、I-4.4 不适用的期望，`INVARIANTS_WITHOUT_A_BAD_IMAGE` 点名 I-4.4；删除线八份镜像的目标名并进 `targets`（只靠名字对接，见 N6）；I-1.10 窄条目宽那条的「跟着转成不适用」表加上 I-3.4（`checker_known_bad_images.rs:3489`）。推翻：干净镜像上这几条的判定与表不同。
- F16 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`：一致。搭建与八份坏镜像的改法（N6）；坏镜像改法都在重封 + 沿引用链补校验和，改的是名字说的那一处（队列根加 7、拿掉 1 号记录、size 加一个净荷、墓碑对象 ID 抬到水位 2、标签 4 值 / 缺行、nlink 改 0）。推翻：某份镜像换种造法之后目标之外的条也红（K2）。
- F17 `crates/singlefs-harness/tests/deletion_scenario_writes.rs`：一致。两条切段单测与两条历史用例（N9）。推翻：四条之一红。
- F18 `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs`：一致（有未钉的格）。十二条用例（`grep -c '#\[test\]'` 现数输出 `12`）：八条「只红自己」、两条中间版、一条读不全、一条每步成立；未钉：中间版的 I-9.5 / I-4.7、读不全的其余缺口。推翻：任一条红。
- F19 `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs`：一致。五张不适用表分别加 5、5、3、5、3 条，新增项名（I-9.5、I-3.4、I-4.7、I-4.6、I-4.4）出现在对的表里（我用脚本抽过每张表里的删除线项名），长度 31 → 36、35 → 40、6 → 9、24 → 29、7 → 10 由编译保证。推翻：该测试在某一步红。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-N1 I-9.5 两个方向 | 一致（有未钉的格） | 两个方向、配对、读不全、没有删除都与 kb、D8 已定项 16 第 3 条、判决逐句对上；方向 ① 的 nlink ≠ 0 一支没有坏镜像也没有变异（kb 已自述） |
| K1-N2 I-4.7 / I-4.6 / I-3.4 | 一致（有未钉的格） | 对象取丙、乙式、标签 4 恒 0 与缺行判红都对；I-4.6 的 size = 0 一支、I-4.7 / I-9.5 的中间版没有用例 |
| K1-N3 判哪几版、配对、读不全 | 一致；extent 树怎么配 = 规则没说 | 版本清单与 I-5.5 同一份；三种情形的报法与 kb 一致；九个缺口变体里七个零用例 |
| K1-N4 下标与 I-4.4 | 一致 | 下标 69–73 依次 I-9.5、I-3.4、I-4.7、I-4.6、I-4.4；理由常量与 kb 叙述差半句，kb 以常量为准；`image.rs` 一处过期注释 |
| K1-N5 I-9.6 墓碑那一半 | 一致（有未钉的格） | 用 I-1.8 同一个已发布谓词、只读所选根的墓碑树；谓词判未发布那一支 CPU 与 GPU 都零用例 |
| K1-N6 坏镜像与用例 | 一致 | 八份镜像各一条 `assert_eq!` 恰好等于目标；登记只靠名字对接；别的条被带成不适用没断言 |
| K1-N7 P1 连带红 | 一致 | I-5.1 与 I-4.7 都红是丙对象的后果；改名与 lint 无行为变化 |
| K1-N8 GPU 内核 | 一致（有未钉的格） | 读代码逐支对 CPU 一支不差；八条对拍的覆盖与漏的格已列；我没跑 GPU |
| K1-N9 E142 导出与删除历史 | 一致 | `--device-bytes` 解析与缺省、H1 / H2 与登记 5.3 / 5.6 逐步对；非 4 GiB 时各区域随盘大小算不算归 K2 |
| K1-N10 E161 装置 | 一致 | I-1 到 I-12 逐条对上，M13–M16 与登记第九节同；没跑装置与内联测试 |
| K3-F1 mutations.tsv | 一致（有未钉的格） | 锚点各命中一次、测试名都在；两行同一处替换；变异没跑 |
| K3-F2 image.rs | 一致 | 74 条、下标、五个常量；过期注释一处 |
| K3-F3 walk.rs | 一致（有未钉的格） | N1–N5 的实现本体；未钉分支见 F3 条 |
| K3-F4 deletion_write_dump.rs | 一致 | 对登记 5.3 / 5.6 第 1 步 |
| K3-F5 e142 导出 bin | 一致 | `--device-bytes` 与零参数缺省 |
| K3-F6 e161 装置 | 一致 | I-1 到 I-12 |
| K3-F7 checker_common.wgsl | 一致 | 下标常量与表偏移、SCRATCH_END 算术对 |
| K3-F8 checker_scan.wgsl | 一致（有未钉的格） | 逐支对 CPU；读不全 / 谓词 / 多层 / 表满未钉 |
| K3-F9 crash_judge_dispatch.rs | 一致 | 测试常数 1797 / 10 782 我算过 |
| K3-F10 crash_judge_gpu.rs | 一致 | 覆盖表 0..=73、SCRATCH_WORDS 298 736 |
| K3-F11 GPU 对拍测试 | 一致（有未钉的格） | 8 条新对拍，6 条只比违例位；没跑（需显卡与 checker 档） |
| K3-F12 scenario.rs | 一致 | H1 / H2 与登记 5.3 逐步对；原函数改成包装、行为不变 |
| K3-F13 abandoned_roots 测试 | 一致 | 只改变量名 |
| K3-F14 cross_links 测试 | 一致 | 断言改成 I-5.1 与 I-4.7 |
| K3-F15 checker_known_bad_images.rs | 一致 | 干净镜像期望与登记；登记只靠名字对接 |
| K3-F16 common_deletion_in_progress/mod.rs | 一致 | 搭建与八份改法对应名字说的那一处 |
| K3-F17 deletion_scenario_writes.rs | 一致 | 四条用例 |
| K3-F18 pool_checker_judges… 测试 | 一致（有未钉的格） | 十二条用例；中间版只有 I-3.4 与 I-4.6，读不全只一种 |
| K3-F19 writable_mount… 测试 | 一致 | 五张表加项名对 |

本轮没有「冲突」格。字面不同但不冲突的三处：I-4.4 理由常量比 kb 叙述少半句（kb 指定常量为准）；`image.rs` 第 86–87 行注释过期；kb 写「标签 4 那一行」而代码判「至少一条设备段 0xFFFF_FFFF 的行、所有标签 4 行值为 0」。

## 没做什么

- 没判 K2（哪一格会错）：没造反例、没在冻结副本拷贝上改任何东西、没跑误红 / 漏判扫描；上面「没钉的格」只是读代码得到的缺口清单，不是反例。
- 没跑任何 cargo、harness 测试、GPU 对拍、变异表、E161 装置与内联测试、E142 / E172 导出：这些要么按共用约束不编译，要么是 checker 档 / 需显卡，归主 agent 提交时。所以「测试红 / 绿」「变异红」我都没有观测，只核了：锚点命中次数、测试名存在、下标常量、算术（草稿区、1797 / 10 782、第一条与第二条流的逐段数）。
- 没复核主 agent 背景里「合入验证 159 个测试二进制全绿」「E142 matched=29 equal=29」「E161 行 1、2 相等」这些实验侧 / 合入侧读数：复核不了（要产物与日志，且不属于这一格）。
- 没读实现员报告 `/tmp/claude-1000/m3-batch18/{x,y,p1,p1fix,p2}/report.md`（只当线索，以代码为准，我直接读代码）；没读禁读清单里的任何文件。
- 没替主 agent 采纳或出判决；「规则没说」的 extent 树配法只是标出来。
- 草稿目录 `/tmp/claude-1000/m3-batch18-code-r1-sonnet/` 里只有两段辅助脚本（`ln.sh` 现查行号、`mutchk.py` 数变异锚点命中）和一段待替换文字，没有编译目录、仓副本或工作树，不需要清理。

## 附：命令原样输出（本轮真跑，未改动）

```
$ cd /tmp/claude-1000/m3-batch18-code-r1/tree; sha256sum -c /home/fy5090/code/singlefs/research/prompts/m3-batch18-code-r1-snapshot/crates-sha256.txt | grep -c ": OK$"
19
$ 同上 | grep -vc ": OK$"
0
$ cd /home/fy5090/code/singlefs; sha256sum -c research/prompts/m3-batch18-code-r1-snapshot/crates-sha256.txt | grep -v ": OK"
crates/mutations.tsv: FAILED
sha256sum: WARNING: 1 computed checksum did NOT match
$ grep -c '^| I-.*已实现' .claude/kb/invariants.md
74
$ python3 mutchk.py | awk "{print \$3}" | sort | uniq -c   # 第 842 与 2602–2627 行原文命中次数
     27 1
$ python3 -c 'w=298736*4; print(w, 2147483644//w, 12*1024**3//w)'
1194944 1797 10782
$ grep -c '#\[test\]' e161 bin / pool_checker 测试
/tmp/claude-1000/m3-batch18-code-r1/tree/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:12
/tmp/claude-1000/m3-batch18-code-r1/tree/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:14
```
