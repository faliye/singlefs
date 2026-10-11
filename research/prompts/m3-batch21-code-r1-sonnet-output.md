# m3-batch21-code-r1 云端正推腿报告（K1、K3）（2026-10-05）

腿：正推（Sonnet）。立场：从条款、判决格与用户定案推到冻结副本里的代码行、测试与变异行，再与被判实现逐格比。
引代码写冻结副本的路径（根 `/tmp/claude-1000/m3-batch21-code-r1/tree`），引 kb 与 prompts 写仓里的路径:行号；行号都是写报告时用 `grep -nF` 式的子串查找现取的。
结论句里标「推的」是读代码推出来的，标「复跑」是 `rerun.sh` 在空目录里跑出来的原样输出，标「复核不了」是我这一档做不到。

## 0　口径、复跑与负载

- 冻结副本 22 份被判文件的 sha256 与 `research/prompts/m3-batch21-code-r1-snapshot/crates-sha256.txt` 逐份相同：复跑第一步 `prepare` 核了（`out/summary.txt` 首行「22 份 sha256 全对」），另在 `mutations.tsv` 的 2759 个数据行上数了：6 列齐、无重名、无重复的原文加替换文、每行原文在源文件里恰好命中 1 次（只有一行指到 `litmus/…`，不在 `crates/` 里，不是这一批的行）。
- 模型目录 `research/prompts/m3-batch21-code-r1-sonnet-model/`：`rerun.sh` 经 `run-with-memory-cap.sh` 编译 6G、跑 4G（显式，不用 auto；峰值表里 6G 超 2 倍，设了 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`），线程上限 10，可续跑。我在空工作目录从头跑了两次调用：第一次到时间退 99，第二次退 0，末行是「全部步骤符合预期（基线全绿、每行变异点名用例红）」。
- 复跑的内容：冻结副本拷贝上编译 core 库与 9 个 harness 测试目标；14 个基线（本批动到的 harness 目标、`singlefs-harness --lib`、慢档）全绿；本批变异表里 harness / core 档的 15 行逐行改坏、整份目标跑、看点名用例红没红；3 个探针（N1 合法全程、N7 上界、N8 两句话）。
- 开跑前 `ps` 看到别的会话在跑 `cargo build --release -p m3dcr2-opus-proto`（6G）、一条 8G 的 `cargo test` 和 `rustc`（`m3-delete-checker-r2-opus` 的目录），没有 `qemu-system`、`vm-bench`、`e152`、`fio`；我的命令都加了 `nice -n 19`，没有等锁（排队时间没单独量）。
- 没跑：checker 档（GPU 对拍、`crash_enumeration_*`）、G 的 11 + 3 行变异、层 0。这些归提交时，见末尾「要主 agent 跑的命令」。

## 1　K1-N1：extent 树那一侧的反方向配对

### 1.1　条款到代码（推的 + 读代码）
- 设计轮判决 F4-b：「定经头 ID 配，配不上记判不了」（`research/prompts/m3-delete-checker-r1-main-verification.md:25`）；第 20 批判决要 extent 树那一侧同样做反方向配对：「S1：extent 树那一侧也做反方向配对」（`research/prompts/m3-batch20-code-r1-main-verification.md:35`）。
- 新成员与理由串：「ExtentTreeHeadIdentifierNamesNoSingleInodeTree,」（`crates/singlefs-checker/src/walk.rs:7943`）；「走读到的某一版树表里有一棵 extent 树，偏移 116 的头 ID 配不上同一张表里恰好一条 inode 树条目」（`crates/singlefs-checker/src/walk.rs:7968`）。
- 判据：每一棵 extent 树取它的头 ID 当 inode 树的树 ID，在同一张树表里数 inode 树条目，不是恰好一条就记判不了：「let inode_trees_named_by_the_extent_tree = tree_table_entries」（`crates/singlefs-checker/src/walk.rs:8484`），「if inode_trees_named_by_the_extent_tree != 1 {」（`crates/singlefs-checker/src/walk.rs:8488`），记在 I-4.7 与 I-4.6 上：「DeletionScanGap::ExtentTreeHeadIdentifierNamesNoSingleInodeTree.reason(),」（`crates/singlefs-checker/src/walk.rs:8495`）。

### 1.2　是不是与 inode 那一侧（第 20 批）同一套两方向

两对配对、各两个方向：
- `deleted_inodes` 条目 ↔ inode 树（第 18 批起）：方向一，每条 `deleted_inodes` 条目按头 ID 找 inode 树，不是恰好一棵就记判不了：「Err(DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree)」（`crates/singlefs-checker/src/walk.rs:8390`）；方向二，每棵 inode 树数指着它的 `deleted_inodes` 条目，不是恰好一条就记判不了：「if deleted_inodes_entries_naming_the_inode_tree != 1 {」（`crates/singlefs-checker/src/walk.rs:8465`）。配不上记在 I-9.5、I-4.7、I-4.6 三条上。
- extent 树 ↔ inode 树：方向一（第 20 批前就有），每棵 inode 树数头 ID 指着它的 extent 树，不是恰好一条就记判不了：「=> Err(DeletionScanGap::ExtentTreeOfTheInodeTreeNotSingle),」（`crates/singlefs-checker/src/walk.rs:8421`）；方向二（本批新加，上面 1.1 的三行），每棵 extent 树数它头 ID 指到的 inode 树，不是恰好一棵就记判不了。配不上只记在 I-4.7、I-4.6 上。

同形：两个反方向都放在正方向那一遍之后，同一不变量上先记的理由留着（「.or_insert(reason);」（`crates/singlefs-checker/src/image.rs:286`）），所以「没有 extent 树指着 inode 树」那一形先记正方向那句更近的理由，代码注释也这么写：「放在上面那一遍之后：没有 extent 树指着」（`crates/singlefs-checker/src/walk.rs:8477`）。
不同之处只有一处，且是被覆盖的：extent 的正方向住在 `deleted_inodes` 条目的循环里，一棵 inode 树没有 `deleted_inodes` 条目指着它时这一遍不跑；这一形由 inode 侧的反方向在三条上都记判不了（「DeletionScanGap::InodeTreeNamedByNoSingleDeletedInodesEntry.reason(),」（`crates/singlefs-checker/src/walk.rs:8469`））。推的：我没找到「某棵 inode 树的 extent 配对既不被正方向也不被任一反方向看到」的树表形态；这个结论由攻方抽样，不由我宣布。

### 1.3　配不上时三条各报什么，与 `invariants.md` 头「判定有四种」
- I-9.5：extent 侧的 gap 只记在 I-4.7、I-4.6 上，代码注释写明：「I-9.5 不读 extent 树，不记。」（`crates/singlefs-checker/src/walk.rs:8478`）。测试在 S1 那一形上钉 I-9.5 照判、成立：「I-9.5 不读 extent 树，照判、成立」（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:656`）（这句话在文件里出现 4 处，第 3 处是 S1 用例的）。
- I-4.7、I-4.6：`part_of_the_range_not_judged` 的归并次序是「违例 > 部分没判 > 这一支 > 成立」：「} else if let Some(reason) = self.part_of_the_range_not_judged.get(invariant) {」（`crates/singlefs-checker/src/image.rs:311`），这一支交回 `NotApplicable(reason)`；所以 gap 压过成立、压不过违例。新循环里只有 `part_of_the_range_not_judged` 一个调用、没有 `judge`，所以违例集合与改动前逐位相同（推的）。
- 与头对得上吗：头写四种判定、不适用必须带理由、不许静默算成立；部分没判「今天只有 I-9.28 …报它」：「今天只有 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）报它」（`.claude/kb/invariants.md:6`）。I-9.5 那一行写明删除态扫描读不全的版报 part_of_the_range_not_judged（即不适用带理由，转述），原句：「这是今天除 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 外各条用的报法」（`.claude/kb/invariants.md:292`）；是否改报第四种判定随 C701 那一问由用户答。新 gap 报「不适用带理由」，理由串非空，与两处**一致**。

### 1.4　冲突与规则没说（两边整行并排）
- 规则没说：`invariants.md` I-9.5 的共用判法只写了 `deleted_inodes` 与 inode 树的配对：「配不上的版记判不了」（`.claude/kb/invariants.md:292`）；extent 树按头 ID 配 inode 树（正反两个方向）与第 18 批加的 inode 树反方向配对，kb 里都没有字。extent 侧配对的依据只有 F4-b 的一般句与第 20 批 S1 的判决，不是 kb 条款。属写回项，不是代码错。
- 冲突（注释对注释，预先存在、本批新 gap 照它的惯例）：
  - 一句：「一版里任一样判不了，三条在那一版都判不了。」（`crates/singlefs-checker/src/walk.rs:7992`）
  - 另一句：「I-9.5 不读 extent 树，不记。」（`crates/singlefs-checker/src/walk.rs:8478`）；实际 extent 侧三种 gap 只记在两条上：「INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,」（`crates/singlefs-checker/src/walk.rs:8490`）。
  - 判：前一句是常量上的概括，extent 侧 gap 今天（含本批新加的）都只记 I-4.7、I-4.6；行为对（I-9.5 不读 extent 树），常量注释说得过满。小出入，不改判。
- 正文 N1 行把箭头写反了：「每棵 extent 树要恰好一棵 inode 树的头 ID 指着它」（`research/prompts/_m3-batch21-code-r1-body.md:13`）；代码与代码注释写的是 extent 树的头 ID 配 inode 树：「每一棵 extent 树的头 ID 都要配上恰好一条 inode 树，」（`crates/singlefs-checker/src/walk.rs:8350`）。以代码为准，是题面措辞，不是实现错。

### 1.5　测试与变异行
- 新用例 「fn an_empty_entry_standing_in_for_the_extent_tree_while_the_real_extent_tree_names_no_inode_tree_leaves_the_data_unit_invariants_not_judged(」（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:623`）：先在合法镜像上钉「照旧报没有对象、不报 extent 树配不上」（「在合法镜像上照旧报没有对象，不报 extent 树配不上」（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:635`）），再在坏镜像上钉 「vec!["I-9.16", "I-9.27"],」（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:647`）（只红这两条）、I-9.5 成立、I-4.7 与 I-4.6 带新理由串（「在真 extent 树配不上 inode 树的那一版判不了，不报没有对象」（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:665`））。
- 坏镜像的改法：livelist 条目（根指针全零）改成种类 1、头 ID 指 inode 树，真 extent 树头 ID 改 0：「pub fn let_the_empty_livelist_entry_stand_in_for_the_extent_tree_in_the_tree_table_at(」（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs:774`）、「livelist 条目的根指针全零（空根）」（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs:796`）、「livelist_entry + TREE_TABLE_ENTRY_KIND_OFFSET,」（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs:805`）、「livelist_entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,」（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs:810`）。它不进 `known_bad_images_*` 那几张登记表：登记表的约定是「改坏之后恰好点名那一条红」，这一形红的是 I-9.16、I-9.27，I-4.7 只报判不了，放不进去，不算漏。
- 复跑：`base-pool_checker_judges` 30 个全绿；变异行 2749（`if false && inode_trees_named_by_the_extent_tree != 1`）整份目标跑只红点名那一条（29 过 1 红），红在 `pool_checker_judges_the_deletion_state_of_every_walked_version.rs:663` 的 I-4.7 理由断言（`out/summary.txt` 的 `mut-2749`）。
- 合法态探针（复跑，`out/probe-n1-lines.txt`）：新池 → 意图 → 批 1、2、3（删完）→ 删完之后建 inode，六个镜像上违例集合全空，I-9.5、I-3.4 成立；意图起 I-4.7、I-4.6 成立（删完那一版也报成立，推的：候选集里还有意图那一版）；意图之前两条报「没有对象」的不适用。新 gap 的理由串在这六个镜像上一次都没出现。
- 推翻 N1「一致」的现象：一份从入口走得到的合法已发布镜像上，I-4.7 或 I-4.6 带 `ExtentTreeHeadIdentifierNamesNoSingleInodeTree` 的理由串；或一份镜像 inode 树与 extent 树的配对两方向都配上而 I-4.7 / I-4.6 对一个单元数不为 0 的号报「没有对象」。

## 2　K1-N2：三处改准

- `walk.rs` 那句：旧句把号的唯一性推给 I-9.4、I-1.1；新句：「同一片叶里同号两条今天没有不变量管」（`crates/singlefs-checker/src/walk.rs:8090`）。对 kb：I-9.4 只写「容器内每条记录的 inode 号 ≥ 容器号；沿叶序容器号严格递增，且左容器的最大 key < 右容器号」：「沿叶序容器号严格递增，且左容器的最大 key < 右容器号」（`.claude/kb/invariants.md:291`），没有容器内记录号严格递增，I-1.1 判的是身份、不是容器内次序（「其头记录的类内身份与实际引用一致」（`.claude/kb/invariants.md:26`）），**一致**。出处是第 20 批判决的 S4a 行：「K2 N1：S4a」（`research/prompts/m3-batch20-code-r1-main-verification.md:26`）。
- `lib.rs` 那句：「新写的地方都调它，不再抄一份」（`crates/singlefs-harness/src/lib.rs:44`）；「checker 档里还有四份装置各自手抄着同样四个名字」（`crates/singlefs-harness/src/lib.rs:45`）。我在冻结副本里数了：四个文件都有手抄的「种类 → 名字」一臂（「RecordedOperationKind::WriteZeroes => "write_zeroes",」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1544`）、「RecordedOperationKind::WriteZeroes => "write_zeroes",」（`crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump_one_device.rs:74`），另两份 e170、e171_after 同形）；「按名字反查种类」的一臂只有前三份有（「"write_zeroes" => RecordedOperationKind::WriteZeroes,」（`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:961`）、「"write_zeroes" => RecordedOperationKind::WriteZeroes,」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:2535`），e171 同形），e142_one_device 没有。注释说「前三份另有一份按名字反查种类的」，**一致**。
- `AccountingTreeUnreadable` 补用例：「fn an_accounting_tree_that_does_not_read_back_leaves_the_pending_deletion_invariant_not_judged() {」（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:673`），把最新根的记账树根（根兼叶）两份整单元清零：「zero_both_copies_of_the_unit_at(&mut image, accounting_root_slot, node_bytes());」（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:676`），断言 I-3.4 报判不了且理由串带「记账树读不全」：「reason.contains(NOT_JUDGED_ACCOUNTING_TREE_UNREADABLE),」（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:679`）。变异行 2750（读不出时 `return Ok(())`，I-3.4 按缺行判红）整份目标跑只红这一条，红在 `reason_not_judged` 里（实际 `Violated(…没有待删除但意图未完成占用…那一行)`，`out/summary.txt` 的 `mut-2750`）。
- 11 种 `DeletionScanGap` 现在每种都有用例：在测试文件里数了每个 `NOT_JUDGED_*` 常量，只有 `InodeTreeUnreadable` 用字面串（「reason.contains("inode 树读不全")」（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:381`）），其余各有常量与用例；**一致**。
- 推翻 N2 的现象：`mut-2750` 那一刀下 `an_accounting_tree_…` 仍绿；或 `DeletionScanGap` 里有一种在测试文件里既没有常量也没有字面串。

## 3　K1-N3：GPU 恢复内核放过上段根层级 0 的零条目 extent 树

### 3.1　条款与判决
- D8 已定项 16 第 5 条：「判定 = 树表有条目 ∧ inode 树零条目根」（`.claude/kb/decisions/08-核心索引结构.md:468`）（原句后半「∧ extent 树零条目根（key 区间按位置规定）∧ `deleted_inodes` 根指针置零」同一行）。
- 第 20 批判决坐实 GPU 缺这一支、改法：「N5：GPU 恢复内核放过零条目 extent 上段根」（`research/prompts/m3-batch20-code-r1-main-verification.md:36`）。

### 3.2　CPU 与 GPU 逐支对（读两边代码；GPU 不跑）
- 零条目只许上段的根兼叶（层级 0）：CPU 「if header.entries.is_empty() && !(is_the_root_of_the_tree && expected_level == 0) {」（`crates/singlefs-core/src/extent_tree.rs:711`）；GPU 「if (n.entry_count == 0u && !(is_the_root_of_the_tree && expected_level == 0u)) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2328`）。**一致**。
- 「是整棵树的根」的取法：CPU 上段 「let is_the_root_of_the_tree = already_read.is_some();」（`crates/singlefs-core/src/extent_tree.rs:1034`）（根是先读好再传进来的那一个），下段传 false；GPU 上段 「level, depth == 0u,」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2367`）（深度 0 只有根），下段 「level, false,」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2533`）。**一致**。
- 上段一条叶条目都没有 ⇒ 读得出、没有数据单元、没有下段：CPU 「if files.is_empty() {」（`crates/singlefs-core/src/recovery.rs:4182`）；GPU 「if (upper_count == 0u) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2688`）。一棵树里非根的叶不许零条目，所以「叶条目总数为 0」恰好等于「根兼叶且零条目」，两边的前提相同（推的）。**一致**。
- 随后：inode 树里没有第一个文件那条记录 ⇒ 没有文件：CPU 「let Some(inode_record) = inode_record else {」（`crates/singlefs-core/src/recovery.rs:5073`）；GPU 「if (!found_inode) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3282`）。nlink = 0 ⇒ 没有文件，且在核单元数之前：CPU 「if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {」（`crates/singlefs-core/src/recovery.rs:5077`）；GPU 「if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3287`）。**一致**。
- inode 树零条目根不拒：CPU 「零条目的 inode 树根是 unlink 删掉树里最后一条记录之后的那一版」（`crates/singlefs-core/src/recovery.rs:2637`）；GPU 只在有条目时核 key 区间 「if (n.entry_count > 0u) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:1692`），层级仍要 1 「if (inode_root.level != 1u) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3218`）；写者写的零条目根就是层级 1：「一片叶都不剩（unlink 删掉树里最后一条记录）⇒ 零条目根」（`crates/singlefs-core/src/transaction.rs:7361`）。**一致**。
- `deleted_inodes` 根指针置零 ⇒ 不计映射节点：CPU 「let deleted_inodes_root_bytes = if deleted_inodes_root_pointer == NodePointer::empty_root() {」（`crates/singlefs-core/src/recovery.rs:2726`）；GPU 「if (pointer.all_zero) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3061`） 之后 「let deletion_tree_nodes = select(0u, 1u, have_deleted_inodes) + container_tree_units;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3193`）。**一致**。
- 本批没动、但同一版会走到的支（读不出的坏盘支、零条目层级 0 的条目宽）：G 交回的「GPU 按条目宽拒、CPU 不核」不在这一轮判（正文第一节「这一轮不判的」），我不重判。

- 推翻 N3「一致」的现象：一个 CPU 答「没有文件」的删完镜像上 GPU 走读失败，或新流的某个状态上 GPU 与 CPU 不一致。

## 4　K1-N4：对拍补一条删到最后一批的流、6 条改名连带 11 行变异、nlink 常量合一

### 4.1　新流是不是真走到最后一批之后的版；11 行变异
- 搭建：「fn record_new_pool_through_the_last_deletion_batch(tag: &str」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:4965`），三个单元按批大小 1 删 「const BATCHES_OF_ONE_THAT_DELETE_A_THREE_UNIT_FILE: usize = 3;」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:4957`） 批，搭建函数自己断言最后一批形状：「last_batch.inode_record.is_none()」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:4973`） 等。同一搭建（`pool_with_a_three_unit_file`、`publish_the_intent`、`publish_the_next_step_in_batches_of_one`）在 harness 侧复跑（`out/probe-n1-lines.txt`）：批 3 后 `inode_record_is_none=true deleted_inodes_is_empty=true data_pointers=0 lower_nodes=0 upper_nodes=1`，批 1、2 不满足，所以那条断言在这个搭建上成立、而且只在第 3 批成立。
- 对拍用例 「fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch() {」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5239`） 用流名 「"deletion-through-the-last-batch",」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5242`），整份比并要求 I-9.5、I-3.4、I-4.7、I-4.6 在 CPU 池级 checker 上判过而成立、不红。
- 用 CPU 算期望、写 GPU 会给什么（推的，GPU 不跑）：最后一批那一版的根上，CPU 冷走读答「没有文件」；GPU 走 extent 树（零条目根层级 0，通过 `judge_extent_node`，`upper_count == 0` 读得出）→ inode 树根层级 1、零条目 → 没有第一个文件那条记录 → `walk_no_file`，与 CPU 同。意图到最后一批之前的版 nlink = 0 ⇒ 都答没有文件。所以预测整份比零不一致。**复核不了**：要 GPU 卡，归提交时。
- 变异行 2754（`if (false && upper_count == 0u)`）、2755（去掉 `judge_extent_node` 里的根兼叶例外）：读代码，两刀都让最后一批那一版在 GPU 上 `walk_failed(32)`、CPU 答没有文件，整份比红。**复核不了**（checker 档）；两行的原文在源文件里各命中 1 次（复跑前数过），点名用例在对拍文件里存在。
- 11 行改名变异：`rename_rows_check.py`（模型目录）把前几批 diff 材料里 8 + 3 行带旧名的变异逐行改名后与冻结副本表里的新行比，原文、替换文、参数、点名用例逐字相同，11 行全 OK，`crates/` 里旧名残留 0 处（`out/rename-rows-check.txt`）。6 条用例的函数名与它们传给 `one_usable_card_or_report_not_run` / `assert_the_gpu_judge_agrees…` 的名字串同步改了（diff 里逐条对过）。
- nlink 常量合一：扫描内核删了自己那份、改读共用那份：「rd32(container.base, record + INODE_RECORD_NLINK_OFFSET)」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2198`）；共用那份 「const INODE_RECORD_NLINK_OFFSET: u32 = 36u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:136`）；扫描内核的源码是共用那一半拼在前面：「{GPU_JUDGE_CHECKER_SCAN_SOURCE}」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:203`）。size 偏移两处定义（「const DELETION_INODE_RECORD_SIZE_OFFSET: u32 = 40u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2068`） 与 「inode_size = rd64(leaf.base, ro + 40u);」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3278`））是 G 交回时留下的下一批项，不判。
- 推翻 N4「一致」的现象：新流某个状态上 GPU 与 CPU 的红位、违例位或恢复结局不同；或一个 CPU 答「没有文件」的删完镜像上 GPU 走读失败（零条目根之外的另一形，K2 抽样）。

## 5　K1-N5：卸载崩在暖机要推两次的那一刀

- 条款：D16 正常卸载一行写明卸载半途崩溃后的下一次可写挂载「同样只放开一截」，其余「扣到这次挂载的暖机在每块盘上都落了根」：「其余扣到这次挂载的暖机在每块盘上都落了根」（`.claude/kb/decisions/16-发布语义.md:41`）。第 19 批判决要补这一刀：「卸载崩在第 15、16 个写之后时暖机要推两次」（`research/prompts/m3-batch19-code-r1-main-verification.md:26`）；改法：「N3 / N4：」（`research/prompts/m3-batch19-code-r1-main-verification.md:38`）…（该行整条）。
- 新用例搭建：覆盖写 「const OVERWRITES_BEFORE_AN_UNMOUNT_WHOSE_NEXT_MOUNT_WARMS_UP_TWICE: usize = 3;」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:668`），卸载落下 「const UNMOUNT_WRITES_LANDED_BEFORE_A_MOUNT_THAT_WARMS_UP_TWICE: u64 = 15;」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:672`） 个写就崩；用例 「fn after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_in_either_warm_up(」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:684`）。
- 断言的是不是名字说的那一件：先钉场景真的是「暖机推两次」「mounted.output.warm_up_publishes.len(),」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:715`） = 2，再钉写行那次与两次暖机各自写下的单元落在「卸载之前现行那一版换下的槽」里的个数都是空：「vec![Vec::<u64>::new(); 3],」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:738`），说明文字 「写行那次、第一次暖机、第二次暖机写下的单元（按这个次序一次一项）」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:739`）。换下的槽是在卸载前取的：「let released_before_the_unmount = slots_released_by_the_current_version(&pool);」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:632`）。**一致**。
- 变异行（复跑，整份目标跑，每行只红一条）：
  - 2757（第一次暖机之后就放开，预演同样放开，不比预演对真发）：只红新用例，红在 `the_mount…rs:736` 的最后一项断言（即上面 `vec![…; 3]` 那条）。
  - 2758（只在真发那一处第一次暖机之后放开，预演照旧落满才放开）：只红新用例，红在核心里预演对真发的那一判 `mount.rs:4908`（变异多插 3 行，原文件在 「取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同」（`crates/singlefs-core/src/mount.rs:4907`） 那条 `assert_eq!`）。与用例自述一致：「挂载在预演对真发那一判上 panic，这一条同样红」。
  - 2759（顺序写那一版的根把 F 带成上一版的 txg）：补的是原第 725 行的三元组断言，现在在 「第一次写一串抬 F 都不推、只发成这一版，F 停在挂载那一刻的值」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:812`）；整份目标只红 `…regrows_it_without_raising_the_floor`，实际 `left: (0, 1, CheckpointTxg(7))`，红在 `the_mount…rs:805`。**一致**。
- 文档三处：N4 用例写明「这个结局只在准入关着时走得到」「这个结局只在准入关着时走得到」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:752`）；暖机只推一次的那条用例指向新用例 「卸载只落下第一个写，这次挂载的暖机只推一次」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:621`）；N3 注释出处改成第 17 批正文第 27 行：「出处：第 17 批正文」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:615`），那一行确有这句：「那一形没有用例钉」（`research/prompts/_m3-batch17-code-r1-body.md:27`）。判别力的边（单元数 8–52、单元区 176/192/240 槽）引的是上一轮攻方报告的 N4 一节，我对了那份报告的判别力的边那一行存在；`n34-176-3-1-a036.txt` 第 150 行确有 `inR=2`（`awk` 读到的）。
- 推翻 N5「一致」的现象：把 `mount::establish_instance` 的放开挪到第一次暖机之后（两处同挪或只挪真发）而这份目标 7 条仍全绿——复跑里 2757、2758 各红 1 条，所以目前没有；卸载崩在别的写点（`UNMOUNT_WRITES_LANDED… ≠ 15`）暖机仍推两次而放开写错的形态，是 K2 的放开扫。

## 6　K1-N6：同一次挂载里两次 unlink、后一次时间更早

- 条款：D8 已定项 6 写入时间一段：「写调用方随这次 unlink 给的时间，三个时间秒（atime、mtime、ctime）共一个值，各批照抄意图那次写的值」（`.claude/kb/decisions/08-核心索引结构.md:195`）。判决要补的一形：「N1：」（`research/prompts/m3-batch19-code-r1-main-verification.md:39`）…（该行：同一次挂载里两次 unlink 后一次更早的用例、qfm 换写法）。
- 新用例 「fn two_session_unlinks_in_the_same_mount_the_second_given_an_earlier_time_write_the_second_given_time_into_the_three_time_fields_of_its_intent_on_disk(」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:823`）：先 unlink 第一个文件（上一版 + 600 秒），再建一个 inode（时间 = 上一版），再 unlink 它（上一版 + 「SECONDS_AFTER_THE_PREVIOUS_VERSION_OF_THE_SECOND_UNLINK_IN_THE_SAME_MOUNT: u64 = 300;」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:804`））；用例自己断言三个时间的次序 「time_of_the_new_inode <= time_given_to_the_second_unlink」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:835`）（建 inode 的时间不晚于后一次给的、后一次早于前一次给的）。断言盘上、两块盘上、仅有的那一版（意图那一版，最后一批已删记录）里 atime、mtime、ctime 秒都是后一次给的值：「vec![vec![[time_given_to_the_second_unlink; 3]; 2]],」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:890`）。**一致**（读的是盘，不是内存里那份记录，D8 条款也是写在盘上的字节）。
- 变异行（复跑，整份目标 23 条，每行只红 1 条）：2760（会话层 unlink 时间取「给的与本线程上一次 unlink 用过的较大者」）只红新用例，红在 `unlink_mount_continuation…rs:888` 的 `assert_eq!`；2761（qfm：本线程建 inode 给过的最晚时间，unlink 取较大者）只红 「fn a_session_unlink_after_a_new_inode_with_a_later_time_in_the_same_mount_writes_the_given_time_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk(」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:788`）（红在 `:795`），新用例在 qfm 那一刀下保持绿——与用例自述 「写下的都是后一次给的时间，照旧绿」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:821`） 一致。旧的「取现行那一版各条记录最晚写入时间」qfm 行已从表里删掉（表里 `Q-F qfm` 只剩 2761 一行，复跑前数过）。
- 推翻 N6「一致」的现象：2760 或 2761 那一刀下整份目标 23 条全绿；或把后一次 unlink 的时间写成给的与前一次较小者而新用例仍绿（K2 放开扫时间次序）。

## 7　K1-N7：C2 的每批换下恰好 min(144, 剩下)，快档 433，慢档 2304、4608

- 条款：D8 已定项 16 第 2 条：「从尾部按数据单元个数分批，每批释放尾部 K 个单元」（`.claude/kb/decisions/08-核心索引结构.md:465`），最后一批释放剩下全部；K 产品取 144。C2 欠账行：「斜率不为 0 判红」（`.claude/kb/checks-owed.md:18`）。判决第 19 批 K1 N6：「N6（并进 C2 欠账那一件）」（`research/prompts/m3-batch19-code-r1-main-verification.md:40`）…。
- 「每批换下恰好 min(K, 剩下)」从条款算出来：「fn data_units_released_by_every_publish_as_the_clause_gives(」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:163`），循环体 「let released_by_this_batch = data_units_remaining.min(data_units_per_batch);」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:172`），意图那次 0 个；用例对每份文件逐项 `assert_eq`（「data_units_released_by_every_publish_as_the_clause_gives(data_units_of_the_file),」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:219`），慢档同一句第 2 处）。条款里最后一批「释放剩下全部」与 `min(K, 剩下)` 在剩下 ≤ K 时同值，剩下恰为 K 的整数倍时最后一批也是 K，**一致**。
- 常数是不是从代码算出来的：「fn write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file() -> u64 {」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:46`） 调 `admission::ring_window_on_one_device`（「let ring_window_in_slots = ring_window_on_one_device(」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:49`）），再用 `ROOT_RING_REGIONS × S + 1`（「let roots_in_the_ring_plus_one = ROOT_RING_REGIONS * slots_per_region.count() + 1;」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:55`））除回 W；W 的式子在代码里是 「/// W：一次删除类发布每盘新占槽数的上界」（`crates/singlefs-core/src/admission.rs:304`）。探针（复跑）打印 `W 式子算的上界：写调用最多的那次至多比一个单元的文件多 748 次`，与背景里的「上界 748」相同：W = 368，2 × (368 + ⌈368 ÷ 64⌉ = 6) = 748（我手算与探针输出一致）。**从代码算**，但盘数是手写的 「const DEVICES_OF_THE_TEST_POOL: u64 = 2;」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:34`），没有断言它等于这个池分配器的盘数；池盘数变了会让上界偏小、用例误红，不会偏大放过（小出入）。
- 绝对值：快档 433 个单元 「vec![33, 39, 43, 39, 29],」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:249`），432 个单元 「vec![33, 37, 35, 33],」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:248`），慢档 「[31, 45, 53],」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:304`）；比一个单元的文件的 31 次多 6、12、14、22，都在 748 之内。基线复跑：快档 + 慢档 2 条全绿（慢档 28.88 秒）。
- 变异行（复跑）：2762（产品批大小取 max(144, ⌈剩下 ÷ 3⌉)）整份目标红快档那一条（433 个单元第一批 145 个，用例自述 「每批取 max(144, ⌈剩下 ÷ 3⌉) 时 433 个单元的文件第一批换下 145 个」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:196`））；2672（`u32::MAX`，整个文件一批）同样只红快档；2763（慢档行，同一刀）红快档与慢档两条。慢档那一行单独看分不开慢档自己的判别力，因为同一刀在 433 就红了；慢档独有的是 2304、4608 两点（批数 16、32）。
- 局限（不改判，进 C2 欠账那一件）：七个规模是有限点。「批数封顶在 32 以上」的实现（每批取 max(144, ⌈剩下 ÷ 32⌉)）在 4608 个单元上仍是 144，七点都过，只在更大的文件上才长；这与正文说的「`斜率为 0` 怎么读」是同一件事，已在「这一轮不判的」里，我只记不判。
- 推翻 N7「一致」的现象：2762 / 2672 / 2763 那一刀下对应用例仍绿；或池分配器盘数 ≠ 2 而上界仍按 2 算（会误红，不是漏红）。

## 8　K1-N8：C479 文件头与变体 c

- 条款：C479 那一行要「再造一个『水位压回去但树里那条记录已被换掉』的镜像，看 I-9.6 还拦不拦得住」：「只供测试的开关把水位压回一个已发过的号」（`.claude/kb/checks-owed.md:384`）；I-9.6 的两半：「统计量 > 该树 inode 树内最大 key」（`.claude/kb/invariants.md:293`），墓碑那一半（2026-10-05 起）。
- 文件头改成「判不出的区域是 p + n 大于两处最大号」：「p + n 大于 checker 判的两处最大号」（`crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:6`）。对代码：树内最大 key 是遍历全部候选根时的运行最大值 「self.largest_inode_number_in_the_inode_tree = Some(」（`crates/singlefs-checker/src/walk.rs:2395`），墓碑那一半取所选根墓碑树 「judgements.judge("I-9.6", watermark > largest_tombstone_object, || {」（`crates/singlefs-checker/src/walk.rs:9392`），所以头里「各条根里 inode 树的最大 key」与「所选根墓碑树里已发布墓碑的最大对象 ID」是对的。
- 探针（复跑，`rerun.py` 从入库用例文件机械剪出前半、接 `probe_n8_tail.rs.txt`）：压回 2、一次建两个（p + n = 4，两处最大号 3 与 3）：`PROBE-N8-TWO watermark=4 birth_of_2=Some(5) birth_of_3=Some(5) I-9.6=Holds violated=[]`——2、3 号都被重发而 I-9.6 成立、整池无违例，文件头那句「判不出的区域」成立，且 n ≥ 2 时压回的号（2）低于发过的最大号（3）。
- 变体 c 用例：「fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant(」（`crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:264`）；搭建 「fn pool_whose_inode_three_is_unlinked_and_whose_floor_is_raised_past_its_record(」（`crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:215`） 里先抬 F 并断言 「pool.output.root.rollback_floor >= txg_of_the_last_deletion_step,」（`crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:248`）（还带 3 号记录的根都低于 F，不进候选集）；断言 I-9.6 违例说明是墓碑那一半 「detail.starts_with("记账里的 inode 号水位 3 不大于所选根")」（`crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:284`）。不抬 F 的对照（探针，复跑）：`PROBE-N8-UNRAISED watermark=3 floor=CheckpointTxg(0) I-9.6=Violated("记账里的 inode 号水位 3 不大于 inode 树内最大 key 3") violated=["I-9.6"]`——意图那条根还在候选集里，红的是树那一半，与用例注释 “不抬 F 时…树内最大 key 那一半先判出 3” 相同，所以抬 F 是把红从树那一半挪到墓碑那一半所必需的。**一致**。
- 变异行 2764（墓碑那一半恒真：`watermark > largest_tombstone_object` → `true`）：复跑整份目标 2 条，红 1 条，红在 `:291` 的 “应当判红，实际 Holds”；另一条（树那一半的用例）保持绿。**一致**。
- 推翻 N8「一致」的现象：2764 那一刀下变体 c 仍绿；或不抬 F 时红的是墓碑那一半（说明抬 F 没起作用）。

## 9　K1-N9：开关遇没有上一版的发布当场 panic

- 条款：C479 那一行没有写新池新建文件这一格；这一格来自第 21 批 W 的派发（`records/2026-10-01-删除与并发事务.md` 第 893 行「C479 新池新建文件时开关不管那一格加断言」），第 19 批判决 K1 N7 记过两处读点同开关、同上一版、同在落盘之前。判：规则没说（kb），按派发与判决读。
- 代码：新方法 「pub fn watermark_of_a_publish_without_a_previous_version(self) -> u64 {」（`crates/singlefs-core/src/transaction.rs:287`），开关那一臂 「InodeNumberWatermarkSource::PressedBackForTests(pressed_back) => panic!(」（`crates/singlefs-core/src/transaction.rs:290`）；发布路径读点两处：建 inode 的起号 「.watermark_this_publish_starts_from(first_new_inode_number);」（`crates/singlefs-core/src/transaction.rs:5909`）（上一版是函数入参，必有），记账里写的水位在 `publish_admitted` 里按 `Some / None` 分：「.watermark_of_a_publish_without_a_previous_version(),」（`crates/singlefs-core/src/transaction.rs:8034`）（`None` 臂，即新池新建文件）。panic 的条件 = 开关装着 ∧ 没有上一版，与「两处读点」的分工对得上：有上一版的读点读 `watermark_this_publish_starts_from`，没有上一版的读 `watermark_of_a_publish_without_a_previous_version`，不存在第三个读点（`set_inode_number_watermark_source` 的调用点只有两份测试文件，复跑前 grep 过）。**一致**。
- 用例：core 库 2 条 「fn a_publish_without_a_previous_version_starts_the_inode_number_watermark_at_two_on_the_product_path(」（`crates/singlefs-core/src/transaction.rs:9458`）、「fn a_pressed_back_inode_number_watermark_on_a_publish_without_a_previous_version_is_a_misused_switch(」（`crates/singlefs-core/src/transaction.rs:9470`），harness 1 条 「fn a_pressed_back_inode_number_watermark_on_the_new_pool_file_creation_panics_before_any_write() {」（`crates/singlefs-harness/tests/inode_number_watermark_switch_on_a_publish_without_a_previous_version.rs:23`），用 `catch_unwind`，并钉「panic 在任何写与屏障之前」「stream.operation_count(),」（`crates/singlefs-harness/tests/inode_number_watermark_switch_on_a_publish_without_a_previous_version.rs:93`）（录制流长度不变）。
- 变异行（复跑）：2751（panic 一臂改成交回产品水位）红 core 库那条 `should_panic`；2752（发布路径不读开关、照旧写死 2）只红 harness 那条，红在 `:78` 的 “应当当场 panic，却交回了 Ok(CheckpointTxg(3))”；2753（产品臂起号写成 1）红 core 库那条产品路径用例。三行各一条、互不重叠。
- 推翻 N9「一致」的现象：开关装着发新池新建文件而不 panic、或 panic 前录制流已多一步；或 `PoolWriter` 上有第三个读开关的发布路径。
## 10　K1-N10：「四个固定点单元」那一族旧说法

- 登记：写行一行 「= 9 单元 × 2 盘 = 18」（`.claude/kb/layout/01-first-txn.md:413`）；暖机 / 抬 F 空发布 「= 8 单元 × 2 盘 = 16」（`.claude/kb/layout/01-first-txn.md:414`）；普通发布（新池新建文件）「t1–t12 单元 × 2 盘 = 24」（`.claude/kb/layout/01-first-txn.md:411`）（= 4 个文件角色 + 8 个固定点单元，各两盘）。所以「写行每盘 9 个单元（实例表 1 + 固定点 8）、暖机与抬 F 空发布每盘 8 个」。
- 逐处对（读注释 + 复跑基线，钉值断言本批一个没动）：
  - 「每盘 9 个单元）」（`crates/singlefs-harness/tests/write_accounting_per_structure.rs:203`）：算术 (32768 + 8 × 16384) × 2 盘 = 327 680 字节、18 次，加 5 次 16 896 字节 = 「calls_and_bytes(23, 344_576),」（`crates/singlefs-harness/tests/write_accounting_per_structure.rs:455`）；暖机 8 × 16384 × 2 = 262 144 + 16 896 = 「calls_and_bytes(21, 279_040),」（`crates/singlefs-harness/tests/write_accounting_per_structure.rs:461`），消息串 「第 {} 次暖机：固定点 8 个单元」（`crates/singlefs-harness/tests/write_accounting_per_structure.rs:462`）。数都对得上，3 条基线绿。
  - 「写行发布重写实例表 + 固定点 8 个单元」（`crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs:200`）：该断言的期望向量 = 实例表 1 + 分配记录树根之下 4 个节点 + 4 个固定点角色的根 = 9，下一行 「assert_eq!(row.record.named.len(), 9」（`crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs:202`）；释放个数 9、8、12 的断言值没改（1 + 8、8、4 + 8），7 条基线绿。
  - 模型：「固定点合计 8 个单元」（`crates/singlefs-harness/src/model.rs:147`）、「树表不是 0 条时的空发布（暖机、抬 F）：固定点单元」（`crates/singlefs-harness/src/model.rs:157`）；核心：「固定点那几族（分配记录树、记账树、中央映射树各连同根之下的节点，加树表单元）」（`crates/singlefs-core/src/transaction.rs:2807`）、「新池几何下分配记录树 5 个节点 + 记账 + 映射 + 树表 = 8 个，字节表八」（`crates/singlefs-core/src/transaction.rs:4348`）、「四个固定点角色的根（分配记录树、记账树、映射树、树表；根之下的节点不在这里）」（`crates/singlefs-core/src/transaction.rs:9318`）（测试模块里的 `rewritten_roles` 函数体恰好是 inode 根 + 这四个角色，对得上）。
  - 其余几份（`history.rs`、`many_inodes_write_and_read.rs`、`unlink_in_histories…`）只改注释：「带文件的一版上的空发布要重写固定点单元」（`crates/singlefs-harness/src/history.rs:1694`）、「四个固定点角色的根（分配记录树、记账、映射、树表）」（`crates/singlefs-harness/tests/many_inodes_write_and_read.rs:157`）、「写行与暖机都重写固定点那几个单元」（`crates/singlefs-harness/tests/unlink_in_histories_judged_by_the_model.rs:21`）；`many_inodes` 4 条、`unlink_in_histories` 8 条基线绿。
  - `checker_known_bad_images.rs` 的 65 536 → 131 072：「盘 0 上差 131 072 字节」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:4789`）。复跑第 302 行变异（候选集不并进「由记录施加出来的」那一版）下 I-3.1 的原样输出：`盘 0：记账的已分配 Some(1032192)，遍历全部有效根得到 901120`（`out/mut-302-I-3.1.txt`），差 1 032 192 − 901 120 = 131 072 = 8 槽 × 16 384。**一致**。
  - `crash_enumeration_fixed_script_stream.rs` 的注释（只改注释，checker 档，没跑）：「实例 2 的根为最新的那 12 个状态」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:1362`），与 8 个固定点单元的说法同；该文件第 461、493 行另有「10 个固定点单元」是回退 D 那次发布（分配记录树 7 个节点 + 记账 + 映射 + 树表，另加谱系两个单元 = 24），本批没动、与登记表回退一行相容。
- 同族旧说法有没有漏的：在冻结副本 `crates/` 里 grep「四个固定点」，剩下的 10 处里 `commit_generated_units…`（“四个固定点…分配记录树五个节点加三个角色”）、`unreadable_abandoned_root_slot`（“四个固定点（分配记录树五个节点加三个角色，八槽）”）、`writable_mount…`、`model.rs`、`many_inodes…`、`transaction.rs` 的“四个固定点角色”都是在讲四个角色并带着节点数，没有「四个单元」的旧意思。

冲突（改过的那句旁边留着的旧引用，预先存在、本批没引入，两边整行并排）：
- 注释说：「crates/singlefs-core/src/transaction.rs 第 506 行」（`crates/singlefs-harness/src/history.rs:1695`）、「已定项 9 在 .claude/kb/decisions/16-发布语义.md 第 192 行」（`crates/singlefs-harness/src/history.rs:1696`）。
- 今天：「零单元发布（D16（发布语义） 已定项 9」（`crates/singlefs-core/src/transaction.rs:1413`）；「#### 已定项 9：一次空发布也写单元」（`.claude/kb/decisions/16-发布语义.md:225`）。
- 判：两个行号过期（506 → 1413，192 → 225）。W 改了这条注释上半句却没顺带改行号，小出入，进下一批注释修正。
- 推翻 N10「一致」的现象：任何一处被改的数（8、9、131 072）与 layout/01 第八节两行对不上；或基线里钉值断言红。

## 11　K3：22 份被判文件各一格

22 份 sha256 全对（0 节）。复跑列写「绿」指该文件所在目标在 `rerun.sh` 里的基线全绿；「未跑」指 checker 档。

| # | 文件 | 本批改了什么（N） | 判定 | 依据 |
|---|---|---|---|---|
| 1 | `crates/singlefs-checker/src/walk.rs` | 新 gap 与 extent 反方向配对、S4a 注释（N1、N2） | 一致，一处小出入 | 第 1、2 节；复跑 2749、2750 红；`DELETION_QUEUE_SCAN_INVARIANTS` 注释说得过满 |
| 2 | `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` | 删自己的 nlink 偏移常量、读共用那份（N4） | 一致（读） | 共用那份拼在前面；size 偏移两处定义是 G 交回的下一批项；未跑 |
| 3 | `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` | 零条目根放过 + 读第一个文件放过零条叶条目（N3），nlink 注释（N4） | 一致（读） | 第 3 节 3.2 七支逐支对；GPU 未跑 |
| 4 | `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` | 注释（N10） | 一致 | 8 个固定点单元与 layout 对；只改注释；未跑 |
| 5 | `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 新流 + 6 条改名（N4） | 一致（读 + harness 侧同搭建复跑） | 搭建断言在 harness 侧同搭建上成立；11 行改名逐行对；未跑 |
| 6 | `crates/singlefs-core/src/transaction.rs` | `watermark_of_a_publish_without_a_previous_version` 与 2 条库单测（N9），注释（N10） | 一致 | 第 9、10 节；2751–2753 红；core 库 4 条水位相关绿 |
| 7 | `crates/singlefs-harness/src/history.rs` | 一句注释（N10） | 一致，一处小出入 | 新半句对；同一注释里两个行号过期 |
| 8 | `crates/singlefs-harness/src/lib.rs` | `name` 的文档注释（N2） | 一致 | 四份手抄、三份反查，逐份数了 |
| 9 | `crates/singlefs-harness/src/model.rs` | 注释（N10） | 一致 | 8 / 9 / 12 与 layout 对；`--lib` 84 条绿 |
| 10 | `crates/singlefs-harness/tests/checker_known_bad_images.rs` | 注释 131 072（N10） | 一致 | 第 302 行变异下 I-3.1 差 131 072 复跑；89 条绿 |
| 11 | `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` | S1 坏镜像改法 + 常量（N1） | 一致 | 被 S1 用例用、用例绿；不进 `known_bad_images_*` 登记表的缘由见 1.5 |
| 12 | `crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs` | 文件头改准 + 变体 c（N8） | 一致 | 第 8 节；探针两句话；2764 红；2 条绿 |
| 13 | `crates/singlefs-harness/tests/inode_number_watermark_switch_on_a_publish_without_a_previous_version.rs` | 新文件（N9） | 一致 | 1 条绿；2752 红 |
| 14 | `crates/singlefs-harness/tests/many_inodes_write_and_read.rs` | 注释（N10） | 一致 | 4 条绿 |
| 15 | `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs` | S1 用例 + `AccountingTreeUnreadable` 用例（N1、N2） | 一致 | 30 条绿；2749、2750 各只红一条 |
| 16 | `crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs` | 新用例 + N4 文档 + N3 出处（N5） | 一致 | 第 5 节；7 条绿；2757–2759 各只红一条 |
| 17 | `crates/singlefs-harness/tests/unlink_in_histories_judged_by_the_model.rs` | 注释（N10） | 一致 | 8 条绿（44 秒） |
| 18 | `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` | 新用例 + 抽出 `time_seconds_of_an_inode_on_each_device`（N6） | 一致 | 23 条绿；2760、2761 各只红一条 |
| 19 | `crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs` | 恰好 min、433、慢档、常数现算（N7） | 一致，一处小出入 | 快 + 慢绿；2762、2672、2763 红；盘数手写 2 |
| 20 | `crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs` | 断言消息串（N10） | 一致 | 7 条绿，钉值未动 |
| 21 | `crates/singlefs-harness/tests/write_accounting_per_structure.rs` | 注释与消息串（N10） | 一致 | 3 条绿；算术对上 |
| 22 | `crates/mutations.tsv` | 追加 16 行、替换 12 行、删 1 行（N1–N9） | 一致 | 2759 行锚点各命中 1 次、6 列、无重名；11 行改名逐行对；harness / core 档 15 行复跑全红 |

## 12　各格的推翻条件（汇总，细处见各节）

- N1：合法已发布镜像上出现新理由串；或两方向都配上而 I-4.7 / I-4.6 对单元数非 0 的号报「没有对象」。
- N2：`mut-2750` 下新用例仍绿；`DeletionScanGap` 有一种既无常量又无字面串。
- N3 / N4：新流某状态 GPU 与 CPU 不一致；CPU 答「没有文件」的删完镜像上 GPU 走读失败。
- N5：2757 / 2758 / 2759 下目标仍全绿。N6：2760 / 2761 下 23 条仍全绿。N7：2762 / 2672 / 2763 下对应用例仍绿。
- N8：2764 下变体 c 仍绿，或不抬 F 时红的是墓碑那一半。N9：开关装着发新池新建文件不 panic，或 panic 前录制流多一步。
- N10：被改的数与 layout/01 第八节对不上，或钉值断言红。

## 13　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-N1 | 一致（kb 没写 extent 配对；一处注释小出入） | extent 侧两方向与 inode 侧同形，只改理由串不改违例集合；合法全程探针无新理由；2749 只红 1 条 |
| K1-N2 | 一致 | S4a 注释、lib.rs 四份手抄三份反查、`AccountingTreeUnreadable` 用例与 2750 都对 |
| K1-N3 | 一致（读，GPU 未跑） | 七支逐支同 CPU；预测整份比零不一致 |
| K1-N4 | 一致（读，GPU 未跑） | 新流搭建断言在 harness 侧同搭建成立；11 行改名逐行对；nlink 常量合一 |
| K1-N5 | 一致 | 新用例断言写行与两次暖机都不写进换下的槽；2757 / 2758 / 2759 各只红 1 条 |
| K1-N6 | 一致 | 用例钉盘上三个时间都是后一次给的；2760 只被它杀，2761 不杀它 |
| K1-N7 | 一致（一处小出入，一处局限） | 每批 min 逐项比；748 从代码算（盘数手写 2）；有限点测不出批数封顶 ≥ 32 |
| K1-N8 | 一致 | 文件头那句经探针成立；变体 c 红在墓碑那一半，不抬 F 红在树那一半 |
| K1-N9 | 一致 | panic 条件 = 开关装着且没有上一版；两处读点分工清楚；三行变异各杀一条 |
| K1-N10 | 一致（一处过期行号） | 8 / 9 / 12 / 131 072 都对上 layout 与复跑；history.rs 旁的两个旧行号过期 |
| K3 | 22 份各一格，全部「一致」，3 份带小出入 | sha256 全对；checker 档 5 份只读 |

## 14　没做什么

- 没判 K2（哪一格会错）：那是攻方腿。N1 的交叉指 / 一棵树被两条条目指着、N3 零条目根之外的另一形、N5 放开扫、N7 错实现，都只写了推翻条件，没有造反例。
- 没跑 checker 档：GPU 对拍（含新流）、G 的 11 + 3 行变异、`crash_enumeration_*`；新流耗时没量。GPU 侧的结论都是读代码的推论。
- 没判「这一轮不判的」那几件：G 的坏盘分叉与 size 偏移两处定义、C2 更紧的常数与斜率读法、三·3 九格与两问。
- 没替主 agent 采纳、写判决或写回 kb（kb 里 I-9.5 缺 extent 配对的字是写回项，只记了观察）。
- 没量排队时间（没出现 252）；没用 `date`，报告里的日期取自派发给我的当天。

## 15　要主 agent 跑的命令（提交时，checker 档，GPU 卡上）

- 对拍文件里新流与 6 条改名用例：`SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/hosts/capped.sh <N> cargo test --offline -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch`（工作目录仓根）；看 `test result` 行，新流应当 0 个状态不一致。
- G 的 14 行变异（2754–2756 与表里 11 行改名行）按门禁 checker-tier-crates-mutation-replay 的整表跑，看每行点名用例红。

## 16　草稿、模型与清理

- 模型：`research/prompts/m3-batch21-code-r1-sonnet-model/`（`rerun.sh`、`rerun.py`、两个探针源 `probe_n1_legit.rs.txt`、`probe_n8_tail.rs.txt`、`rename_rows_check.py`、`out/`）。`rerun.sh` 在空目录里从头调用两次跑通（退 99 再退 0）。
- 草稿 `/tmp/claude-1000/m3-batch21-code-r1-sonnet/`：`work/`（冻结副本拷贝、`target/` 编译目录、日志）交回前删掉；`gen/`（报告生成脚本）、`rowcheck*.py` 留在草稿目录，不是编译目录也不是仓副本。
